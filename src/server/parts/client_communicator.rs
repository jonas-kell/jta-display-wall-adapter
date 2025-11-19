use crate::args::Args;
use crate::instructions::InstructionCommunicationChannel;
use crate::interface::{MessageFromClientToServer, MessageFromServerToClient, ServerStateMachine};
use futures::prelude::*;
use std::io::{self, Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{self, sleep};
use tokio_serde::formats::Bincode;
use tokio_serde::Framed;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub async fn client_communicator(
    args: Args,
    server_state: Arc<Mutex<ServerStateMachine>>,
    comm_channel: InstructionCommunicationChannel,
    shutdown_marker: Arc<AtomicBool>,
    client_addr: SocketAddr,
) -> io::Result<()> {
    let server_state_exchange = server_state.clone();
    let shutdown_marker_exchange = shutdown_marker.clone();
    let comm_channel_client_client_exchange = comm_channel.clone();
    let client_exchange_task = tokio::spawn(async move {
        loop {
            if shutdown_marker_exchange.load(Ordering::SeqCst) {
                debug!("Shutdown requested, stopping listener on {}", client_addr);
                break;
            }

            // Wait for new connection with timeout so we can check shutdown flag periodically
            match time::timeout(
                Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
                TcpStream::connect(client_addr),
            )
            .await
            {
                Ok(Ok(client_stream)) => {
                    debug!("Connected to client at {}", client_addr);

                    // on connection first request version to initiate communication
                    {
                        let mut guard = server_state_exchange.lock().await;
                        guard.make_server_request_client_version().await;
                        debug!("Requested server version from client {}", client_addr);
                    }

                    // handle messaging
                    let (read_half, write_half) = client_stream.into_split();
                    let mut deserializer: Framed<
                        _,
                        MessageFromClientToServer,
                        MessageFromServerToClient,
                        _,
                    > = Framed::new(
                        FramedRead::new(read_half, LengthDelimitedCodec::new()),
                        Bincode::<MessageFromClientToServer, MessageFromServerToClient>::default(),
                    );
                    let mut serializer: Framed<
                        _,
                        MessageFromClientToServer,
                        MessageFromServerToClient,
                        _,
                    > = Framed::new(
                        FramedWrite::new(write_half, LengthDelimitedCodec::new()),
                        Bincode::<MessageFromClientToServer, MessageFromServerToClient>::default(),
                    );

                    let shutdown_marker_read = shutdown_marker_exchange.clone();
                    let server_state_read = server_state_exchange.clone();

                    let read_handler = async move {
                        loop {
                            if shutdown_marker_read.load(Ordering::SeqCst) {
                                debug!(
                                    "Shutdown marker set, breaking main client -> self transfer"
                                );
                                break;
                            }

                            match time::timeout(
                                Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
                                deserializer.next(),
                            )
                            .await
                            {
                                Err(_) => {
                                    trace!(
                                        "No new TCP traffic from client within timeout interval"
                                    );
                                    continue;
                                }
                                Ok(None) => return Err("Client TCP stream went away".into()),
                                Ok(Some(Err(e))) => return Err(e.to_string()),
                                Ok(Some(Ok(mes))) => {
                                    // message from server
                                    let mut guard = server_state_read.lock().await;
                                    guard.parse_client_command(mes).await;
                                }
                            }
                        }
                        Ok::<_, String>(())
                    };

                    let shutdown_marker_write = shutdown_marker_exchange.clone();
                    let comm_channel_client_outbound_write =
                        comm_channel_client_client_exchange.clone();

                    let write_handler = async move {
                        loop {
                            if shutdown_marker_write.load(Ordering::SeqCst) {
                                debug!(
                                    "Shutdown marker set, breaking main self -> client transfer"
                                );
                                break;
                            }

                            match comm_channel_client_outbound_write
                                .wait_for_command_to_send_to_client()
                                .await
                            {
                                Err(_) => {
                                    trace!(
                                        "No new command to send to client within timeout interval"
                                    );
                                    continue;
                                }
                                Ok(Err(e)) => return Err(e.to_string()),
                                Ok(Ok(msg)) => match serializer.send(msg).await {
                                    Ok(()) => {
                                        trace!("Communication to client was sent out");
                                        continue;
                                    }
                                    Err(e) => return Err(e.to_string()),
                                },
                            }
                        }

                        Ok::<_, String>(())
                    };

                    match tokio::try_join!(read_handler, write_handler) {
                        Ok(_) => (),
                        Err(e) => {
                            error!("Client connection gone away: {}", e);
                            sleep(Duration::from_millis(1000)).await; // on dev the communication goes into docker, so it connects, then fails. But this spams logs. Slow down retry a bit
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("Client exchange error: {}", e);
                    sleep(Duration::from_millis(1000)).await; // on missing target the communication sometimes connects with "Error - connection refused" -> immediately fails. But this spams logs. Slow down retry a bit
                }
                Err(_) => {
                    // expected on timeout, just loop
                    trace!(
                        "No TCP connection to client could be established within timeout interval"
                    );
                }
            }
        }

        Ok::<_, Error>(())
    });

    // TODO: this tecnically does not need its own channel AND is kind of misplaced in this worker. -> should have at leats its own worker
    let server_state_intake = server_state;
    let comm_channel = comm_channel;
    let shutdown_marker_intake = shutdown_marker;
    let intake_commands_task = tokio::spawn(async move {
        loop {
            if shutdown_marker_intake.load(Ordering::SeqCst) {
                debug!("Shutdown requested, stopping client communicator");
                break;
            }

            match comm_channel.wait_for_incomming_command().await {
                Ok(command_res) => match command_res {
                    Ok(comm) => {
                        let mut guard = server_state_intake.lock().await;
                        guard.parse_incoming_command(comm).await;
                    }
                    Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
                },
                Err(_) => {
                    trace!("No incoming command to report in timeout interval");
                    continue;
                }
            };
        }
        Ok::<_, Error>(())
    });

    let (a, b) = tokio::try_join!(client_exchange_task, intake_commands_task)?;
    a?;
    b?;

    Ok(())
}
