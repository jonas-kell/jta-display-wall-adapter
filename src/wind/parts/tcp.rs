use crate::args::Args;
use crate::wind::format::{make_json_exchange_codec, MessageToWindServer, WindMessageBroadcast};
use crate::wind::parts::wind_state_management::WindStateManager;
use async_broadcast::InactiveReceiver;
use futures::prelude::*;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time;
use tokio_serde::{formats::Json, Framed};
use tokio_util::codec::{FramedRead, FramedWrite};

pub async fn run_network_task(
    args: Args,
    listen_addr: SocketAddr,
    rx_from_com_port: InactiveReceiver<WindMessageBroadcast>,
    shutdown_marker: Arc<AtomicBool>,
) -> Result<(), Error> {
    let listener = TcpListener::bind(listen_addr).await?;
    info!("TCP listener started on {}", listen_addr);

    loop {
        if shutdown_marker.load(Ordering::SeqCst) {
            info!("Shutdown requested, stopping listener on {}", listen_addr);
            break;
        }

        // Wait for new connection with timeout so we can check shutdown flag periodically
        match time::timeout(
            Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
            listener.accept(),
        )
        .await
        {
            Ok(Ok((inbound, client_addr))) => {
                info!("Accepted connection from {}", client_addr);

                let (read_half, write_half) = inbound.into_split();
                let mut deserializer: Framed<_, MessageToWindServer, WindMessageBroadcast, _> =
                    Framed::new(
                        FramedRead::new(read_half, make_json_exchange_codec()),
                        Json::<MessageToWindServer, WindMessageBroadcast>::default(),
                    );
                let mut serializer: Framed<_, MessageToWindServer, WindMessageBroadcast, _> =
                    Framed::new(
                        FramedWrite::new(write_half, make_json_exchange_codec()),
                        Json::<MessageToWindServer, WindMessageBroadcast>::default(),
                    );

                // Connection is accepted. Handle all further in own task

                let shutdown_marker = shutdown_marker.clone();
                let mut rx_from_com_port = rx_from_com_port.activate_cloned();

                // state manager
                let wind_state_manager = Arc::new(Mutex::new(WindStateManager::new()));

                tokio::spawn(async move {
                    let shutdown_marker_read = shutdown_marker.clone();
                    let wind_state_manager_read = wind_state_manager.clone();

                    let read_handler = async move {
                        loop {
                            if shutdown_marker_read.load(Ordering::SeqCst) {
                                debug!(
                                    "Shutdown marker set, breaking wind server inbound transfer"
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
                                    trace!("No new TCP traffic within timeout interval");
                                    continue;
                                }
                                Ok(None) => return Err("TCP stream went away".into()),
                                Ok(Some(Err(e))) => return Err(e.to_string()),
                                Ok(Some(Ok(mes))) => match mes {
                                    MessageToWindServer::SetTime(dt) => {
                                        let mut wsm = wind_state_manager_read.lock().await;
                                        wsm.update_internal_time(dt);
                                    }
                                },
                            }
                        }
                        Ok::<_, String>(())
                    };

                    let shutdown_marker_write = shutdown_marker;
                    let wind_state_manager_write = wind_state_manager.clone();

                    let write_handler = async move {
                        loop {
                            if shutdown_marker_write.load(Ordering::SeqCst) {
                                debug!(
                                    "Shutdown marker set, breaking wind server outbound transfer"
                                );
                                break;
                            }

                            match time::timeout(
                                Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
                                rx_from_com_port.recv(),
                            )
                            .await
                            {
                                Err(_) => {
                                    trace!("No new Messages to send out within timeout interval");
                                    continue;
                                }
                                Ok(Err(e)) => return Err(e.to_string()),
                                Ok(Ok(mes)) => {
                                    // populate the time and type field
                                    let mes = {
                                        let mut wsm = wind_state_manager_write.lock().await;
                                        wsm.populate_broadcast_message(mes)
                                    };

                                    match serializer.send(mes).await {
                                        Ok(()) => trace!(
                                            "TCP sender forwarded message from internal comm channel"
                                        ),
                                        Err(e) => return Err(e.to_string()),
                                    };
                                }
                            }
                        }

                        Ok::<_, String>(())
                    };

                    tokio::try_join!(read_handler, write_handler)?;

                    Ok::<_, String>(())
                });
            }
            Ok(Err(e)) => error!("Accept error: {}", e),
            Err(_) => {
                // expected on timeout, just loop
                trace!("No new TCP connection within timeout interval");
            }
        }
    }

    Ok(())
}
