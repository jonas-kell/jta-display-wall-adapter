use crate::args::Args;
use crate::interface::{MessageFromClientToServer, MessageFromServerToClient, ServerStateMachine};
use crate::server::comm_channel::InstructionCommunicationChannel;
use futures::prelude::*;
use std::io;
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
    loop {
        if shutdown_marker.load(Ordering::SeqCst) {
            info!("Shutdown requested, stopping listener on {}", client_addr);
            break;
        }
        {
            let mut guard = server_state.lock().await;
            guard.set_main_display_state(false);
        }

        // Wait for new connection with timeout so we can check shutdown flag periodically
        match time::timeout(
            Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
            TcpStream::connect(client_addr),
        )
        .await
        {
            Ok(Ok(client_stream)) => {
                info!("Connected to client at {}", client_addr);

                // on connection first request version to initiate communication
                {
                    let mut guard = server_state.lock().await;
                    guard.set_main_display_state(true);
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

                let shutdown_marker_read = shutdown_marker.clone();
                let server_state_read = server_state.clone();

                let read_handler = async move {
                    loop {
                        if shutdown_marker_read.load(Ordering::SeqCst) {
                            debug!("Shutdown marker set, breaking main client -> self transfer");
                            break;
                        }

                        match time::timeout(
                            Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
                            deserializer.next(),
                        )
                        .await
                        {
                            Err(_) => {
                                trace!("No new TCP traffic from client within timeout interval");
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

                let shutdown_marker_write = shutdown_marker.clone();
                let comm_channel_client_outbound_write = comm_channel.clone();

                let write_handler = async move {
                    loop {
                        if shutdown_marker_write.load(Ordering::SeqCst) {
                            debug!("Shutdown marker set, breaking main self -> client transfer");
                            break;
                        }

                        match comm_channel_client_outbound_write
                            .wait_for_command_to_send_to_client()
                            .await
                        {
                            Err(_) => {
                                trace!("No new command to send to client within timeout interval");
                                continue;
                            }
                            Ok(Err(e)) => return Err(e.to_string()),
                            Ok(Ok(msg)) => match serializer.send(msg).await {
                                Ok(()) => {
                                    // trace!("Communication to client was sent out"); // during timing this is even too much for tracing
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
                        {
                            let mut guard = server_state.lock().await;
                            guard.set_main_display_state(false);
                        }
                        sleep(Duration::from_millis(1000)).await; // on dev the communication goes into docker, so it connects, then fails. But this spams logs. Slow down retry a bit
                    }
                }
            }
            Ok(Err(e)) => {
                error!("Client exchange error: {}", e);
                {
                    let mut guard = server_state.lock().await;
                    guard.set_main_display_state(false);
                }
                sleep(Duration::from_millis(1000)).await; // on missing target the communication sometimes connects with "Error - connection refused" -> immediately fails. But this spams logs. Slow down retry a bit
            }
            Err(_) => {
                // expected on timeout, just loop
                trace!("No TCP connection to client could be established within timeout interval");
                {
                    let mut guard = server_state.lock().await;
                    guard.set_main_display_state(false);
                }
            }
        }
    }

    Ok(())
}
