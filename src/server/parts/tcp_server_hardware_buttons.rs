use crate::args::Args;
use crate::hardware_button_exchange_format::{
    HardwareButtonStateBroadcast, MessageFromHardwareButton,
};
use crate::interface::ServerStateMachineServerStateReader;
use crate::json::make_json_exchange_codec;
use crate::server::comm_channel::InstructionCommunicationChannel;
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time;
use tokio_serde::{formats::Json, Framed};
use tokio_util::codec::{FramedRead, FramedWrite};

pub async fn tcp_server_hardware_buttons(
    args: Args,
    state_reader: ServerStateMachineServerStateReader,
    comm_channel: InstructionCommunicationChannel,
    shutdown_marker: Arc<AtomicBool>,
    hardware_buttons_listen_address: Option<SocketAddr>,
) -> std::io::Result<()> {
    let hardware_buttons_listen_address =
        if let Some(hardware_buttons_listen_address) = hardware_buttons_listen_address {
            hardware_buttons_listen_address
        } else {
            // never open hardware buttons tcp server -> we can just die
            return Ok(());
        };

    let listener = TcpListener::bind(hardware_buttons_listen_address).await?;
    info!(
        "TCP listener for hardware buttons started on {}",
        hardware_buttons_listen_address
    );

    loop {
        if shutdown_marker.load(Ordering::SeqCst) {
            info!(
                "Shutdown requested, stopping hardware button listener on {}",
                hardware_buttons_listen_address
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
            listener.accept(),
        )
        .await
        {
            Ok(Ok((inbound, client_addr))) => {
                info!("Accepted hardware button connection from {}", client_addr);

                let (read_half, write_half) = inbound.into_split();
                let mut deserializer: Framed<
                    _,
                    MessageFromHardwareButton,
                    HardwareButtonStateBroadcast,
                    _,
                > = Framed::new(
                    FramedRead::new(read_half, make_json_exchange_codec()),
                    Json::<MessageFromHardwareButton, HardwareButtonStateBroadcast>::default(),
                );
                let mut serializer: Framed<
                    _,
                    MessageFromHardwareButton,
                    HardwareButtonStateBroadcast,
                    _,
                > = Framed::new(
                    FramedWrite::new(write_half, make_json_exchange_codec()),
                    Json::<MessageFromHardwareButton, HardwareButtonStateBroadcast>::default(),
                );

                // Connection is accepted. Handle all further in own task

                let shutdown_marker = shutdown_marker.clone();
                let comm_channel = comm_channel.clone();

                tokio::spawn(async move {
                    let shutdown_marker_read = shutdown_marker.clone();
                    let comm_channel_read = comm_channel.clone();

                    let read_handler = async move {
                        loop {
                            if shutdown_marker_read.load(Ordering::SeqCst) {
                                debug!(
                                    "Shutdown marker set, breaking hardware button -> self transfer"
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
                                    trace!("No new TCP traffic within timeout interval (hardware button)");
                                    continue;
                                }
                                Ok(None) => {
                                    return Err("TCP stream went away (hardware button)".into())
                                }
                                Ok(Some(Err(e))) => return Err(e.to_string()),
                                Ok(Some(Ok(mes))) => {
                                    match comm_channel_read
                                        .take_in_command_from_hardware_button(mes)
                                    {
                                        Ok(()) => trace!("Message from hardware button taken into internal communication"),
                                        Err(e) =>  {
                                            error!("Hardware Button listener server could not deposit message into internal comm channel: {}", e.to_string());
                                            // problems with the internal comm channel (technically this is reason to crash on the spot, this is kind of not supported by the err architecture ins this case -> other places will shut down the program if this happens)
                                            return Err(e.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        Ok::<_, String>(())
                    };

                    let shutdown_marker_write = shutdown_marker;
                    let mut hardware_button_server_receiver =
                        comm_channel.hardware_button_receiver();

                    let write_handler = async move {
                        loop {
                            if shutdown_marker_write.load(Ordering::SeqCst) {
                                debug!(
                                    "Shutdown marker set, breaking self -> hardware button transfer"
                                );
                                break;
                            }

                            match hardware_button_server_receiver.wait_for_some_data().await {
                                Ok(Ok(mes)) => match serializer.send(mes).await {
                                    Ok(()) => trace!("Broadcast to hardware buttons was emitted"),
                                    Err(e) => {
                                        error!(
                                            "Error in outbound hardware button communication: {}",
                                            e.to_string()
                                        );
                                        // will attempt to reconnect in next iteration
                                        return Err(e.to_string());
                                    }
                                },
                                Ok(Err(e)) => {
                                    error!(
                                        "Hardware Button server could not read from internal comm channel: {}",
                                        e.to_string()
                                    );
                                    // problems with the internal comm channel (technically this is reason to crash on the spot, this is kind of not supported by the err architecture ins this case -> other places will shut down the program if this happens)
                                    return Err(e.to_string());
                                }
                                Err(_) => {
                                    trace!(
                                        "No outbound hardware button state message within timeout interval"
                                    );
                                    continue;
                                }
                            }
                        }

                        Ok::<_, String>(())
                    };

                    match tokio::try_join!(read_handler, write_handler) {
                        Ok(_) => (),
                        Err(e) => {
                            error!("Hardware Button listening task crashed unexpectedly: {}", e);
                        }
                    }

                    Ok::<_, String>(())
                });
            }
            Ok(Err(e)) => error!("Accept error: {}", e),
            Err(_) => {
                // expected on timeout, just loop
                trace!(
                    "No new TCP connection (to hardware button interface) within timeout interval"
                );
            }
        }
    }

    Ok(())
}
