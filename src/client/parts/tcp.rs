use crate::args::Args;
use crate::interface::{MessageFromClientToServer, MessageFromServerToClient};
use async_broadcast::InactiveReceiver;
use async_channel::{Sender, TrySendError};
use futures::prelude::*;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time;
use tokio_serde::formats::*;
use tokio_serde::Framed;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub async fn run_network_task(
    args: Args,
    tx_to_ui: Sender<MessageFromServerToClient>,
    rx_from_ui: InactiveReceiver<MessageFromClientToServer>,
    shutdown_marker: Arc<AtomicBool>,
) -> Result<(), Error> {
    let listen_addr: SocketAddr = format!("0.0.0.0:{}", args.display_client_communication_port)
        .parse()
        .expect("Invalid internal communication address");

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
                let mut deserializer: Framed<
                    _,
                    MessageFromServerToClient,
                    MessageFromClientToServer,
                    _,
                > = Framed::new(
                    FramedRead::new(read_half, LengthDelimitedCodec::new()),
                    Bincode::<MessageFromServerToClient, MessageFromClientToServer>::default(),
                );
                let mut serializer: Framed<
                    _,
                    MessageFromServerToClient,
                    MessageFromClientToServer,
                    _,
                > = Framed::new(
                    FramedWrite::new(write_half, LengthDelimitedCodec::new()),
                    Bincode::<MessageFromServerToClient, MessageFromClientToServer>::default(),
                );

                // Connection is accepted. Handle all further in own task

                let shutdown_marker = shutdown_marker.clone();
                let tx_to_ui = tx_to_ui.clone();
                let mut rx_from_ui = rx_from_ui.activate_cloned();

                tokio::spawn(async move {
                    let shutdown_marker_read = shutdown_marker.clone();

                    let read_handler = async move {
                        loop {
                            if shutdown_marker_read.load(Ordering::SeqCst) {
                                debug!(
                                    "Shutdown marker set, breaking main external -> self transfer"
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
                                Ok(Some(Ok(mes))) => match tx_to_ui.try_send(mes) {
                                    Ok(()) => (),
                                    Err(TrySendError::Closed(_)) => {
                                        return Err(format!(
                                            "Internal communication channel closed..."
                                        ))
                                    }
                                    Err(TrySendError::Full(_)) => {
                                        trace!("Internal communication channel is full. Seems like there is no source to consume");
                                    }
                                },
                            }
                        }
                        Ok::<_, String>(())
                    };

                    let shutdown_marker_write = shutdown_marker;

                    let write_handler = async move {
                        loop {
                            if shutdown_marker_write.load(Ordering::SeqCst) {
                                debug!(
                                    "Shutdown marker set, breaking main self -> external transfer"
                                );
                                break;
                            }

                            match time::timeout(
                                Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
                                rx_from_ui.recv(),
                            )
                            .await
                            {
                                Err(_) => {
                                    trace!("No new Messages to send out within timeout interval");
                                    continue;
                                }
                                Ok(Err(e)) => return Err(e.to_string()),
                                Ok(Ok(mes)) => match serializer.send(mes).await {
                                    Ok(()) => trace!(
                                        "TCP sender forwarded message from internal comm channel"
                                    ),
                                    Err(e) => return Err(e.to_string()),
                                },
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
