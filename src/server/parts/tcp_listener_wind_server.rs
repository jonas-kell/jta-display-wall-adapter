use crate::args::Args;
use crate::interface::ServerStateMachineServerStateReader;
use crate::json::make_json_exchange_codec;
use crate::server::comm_channel::InstructionCommunicationChannel;
use crate::wind::format::{MessageToWindServer, WindMessageBroadcast};
use futures::{SinkExt, StreamExt};
use std::io::{self, Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time;
use tokio::time::sleep;
use tokio_serde::{formats::Json, Framed};
use tokio_util::codec::{FramedRead, FramedWrite};

pub async fn tcp_listener_wind_server(
    args: Args,
    state_reader: ServerStateMachineServerStateReader,
    comm_channel: InstructionCommunicationChannel,
    shutdown_marker: Arc<AtomicBool>,
    wind_server_addr: Option<SocketAddr>,
) -> io::Result<()> {
    let wind_server_addr = if let Some(wind_server_addr) = wind_server_addr {
        wind_server_addr
    } else {
        // never listen to the wind server -> we can just die
        return Ok(());
    };

    loop {
        if shutdown_marker.load(Ordering::SeqCst) {
            info!(
                "Shutdown requested, stopping trying to connect to {}",
                wind_server_addr
            );
            break;
        }
        if !state_reader.external_connection_is_allowed().await {
            warn!("Stopped external connection from forming for now");
            time::sleep(Duration::from_millis(1000)).await;
            continue;
        }

        // Wait for new connection with timeout so we can check shutdown flag periodically
        match time::timeout(
            Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
            TcpStream::connect(wind_server_addr),
        )
        .await
        {
            Ok(Ok(timing_stream)) => {
                info!("Connected to wind server target {}", wind_server_addr);

                let (read_half, write_half) = timing_stream.into_split();
                let mut deserializer: Framed<_, WindMessageBroadcast, MessageToWindServer, _> =
                    Framed::new(
                        FramedRead::new(read_half, make_json_exchange_codec()),
                        Json::<WindMessageBroadcast, MessageToWindServer>::default(),
                    );
                let mut serializer: Framed<_, WindMessageBroadcast, MessageToWindServer, _> =
                    Framed::new(
                        FramedWrite::new(write_half, make_json_exchange_codec()),
                        Json::<WindMessageBroadcast, MessageToWindServer>::default(),
                    );

                let args_read = args.clone();
                let shutdown_marker_read = shutdown_marker.clone();
                let comm_channel_read = comm_channel.clone();

                let wind_server_read = async move {
                    loop {
                        if shutdown_marker_read.load(Ordering::SeqCst) {
                            debug!("Shutdown marker set, breaking wind server reading");
                            break;
                        }

                        match time::timeout(
                            Duration::from_millis(args_read.wait_ms_before_testing_for_shutdown),
                            deserializer.next(),
                        )
                        .await
                        {
                            Ok(Some(read_result)) => match read_result {
                                Ok(mess_broadcast) => match comm_channel_read.take_in_command_from_wind_server(mess_broadcast) {
                                    Ok(()) => trace!("Message from wind server taken into internal communication"),
                                    Err(e) =>  {
                                        error!("Wind server could not deposit message into internal comm channel: {}", e.to_string());
                                        // problems with the internal comm channel (technically this is reason to crash on the spot, this is kind of not supported by the err architecture ins this case -> other places will shut down the program if this happens)
                                        return Err(Error::new(ErrorKind::Other, e.to_string()));
                                    }
                                },
                                Err(e) => {
                                    error!("Error in wind server communication: {}", e.to_string());
                                    // will attempt to reconnect in next iteration
                                    return Err(Error::new(ErrorKind::Other, e.to_string()));
                                }
                            },
                            Ok(None) => {
                                let err_mes = "Wind server TCP stream went away";
                                error!("{}", err_mes);
                                return Err(Error::new(ErrorKind::Other, err_mes.to_string()));
                            }
                            Err(_) => {
                                trace!("No TCP message on wind server within timeout interval");
                                continue;
                            }
                        };
                    }

                    Ok::<_, Error>(())
                };

                let shutdown_marker_write = shutdown_marker.clone();
                let mut wind_server_receiver = comm_channel.wind_server_receiver();

                let wind_server_write = async move {
                    loop {
                        if shutdown_marker_write.load(Ordering::SeqCst) {
                            debug!("Shutdown marker set, breaking wind server writing");
                            break;
                        }

                        match wind_server_receiver.wait_for_some_data().await {
                            Ok(Ok(mes)) => match serializer.send(mes).await {
                                Ok(()) => trace!("Message to wind server was emitted"),
                                Err(e) => {
                                    error!(
                                        "Error in outbound wind server communication: {}",
                                        e.to_string()
                                    );
                                    // will attempt to reconnect in next iteration
                                    return Err(Error::new(ErrorKind::Other, e.to_string()));
                                }
                            },
                            Ok(Err(e)) => {
                                error!(
                                    "Wind server could not read from internal comm channel: {}",
                                    e.to_string()
                                );
                                // problems with the internal comm channel (technically this is reason to crash on the spot, this is kind of not supported by the err architecture ins this case -> other places will shut down the program if this happens)
                                return Err(Error::new(ErrorKind::Other, e.to_string()));
                            }
                            Err(_) => {
                                trace!("No outbound wind server message within timeout interval");
                                continue;
                            }
                        }
                    }

                    Ok::<_, Error>(())
                };

                match tokio::try_join!(wind_server_read, wind_server_write) {
                    Err(e) => {
                        error!("Error in a wind server listener task: {}", e.to_string());
                    }
                    Ok(_) => info!("All wind server listeners closed successfully"),
                };

                // only one wind server can be connected at a time -> we continue -> if the jobs exited due to abort, then the next loop wil also exit correctly
            }
            Ok(Err(e)) => {
                error!("Wind server exchange error: {}", e);
                sleep(Duration::from_millis(1000)).await; // on missing target the communication sometimes connects with "Error - connection refused" -> immediately fails. But this spams logs. Slow down retry a bit
            }
            Err(_) => {
                // expected on timeout, just loop
                trace!(
                    "No TCP connection to wind server could be established within timeout interval"
                );
            }
        }
    }

    Ok(())
}
