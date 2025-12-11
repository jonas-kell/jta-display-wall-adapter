use crate::args::Args;
use crate::hex::hex_log_bytes;
use crate::instructions::InstructionToTimingProgram;
use crate::interface::{ServerState, ServerStateMachineServerStateReader};
use crate::server::comm_channel::{
    InstructionCommunicationChannel, PacketCommunicationChannel, PacketData,
};
use crate::server::nrbf::{generate_response_bytes, BufferedParser};
use async_broadcast::RecvError;
use std::io;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::time;

enum TimeoutOrIoError {
    Timeout,
    SendingBlocked,
    ReceiveError(RecvError),
}

pub async fn tcp_listener_timing_program(
    args: Args,
    state_reader: ServerStateMachineServerStateReader,
    comm_channel: InstructionCommunicationChannel,
    comm_channel_packets: PacketCommunicationChannel,
    shutdown_marker: Arc<AtomicBool>,
    listen_addr: SocketAddr,
) -> io::Result<()> {
    // never listen to the timing program -> we can just die
    if !args.listen_to_timing_program {
        return Ok(());
    };

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

                let (mut ri, mut wi) = inbound.into_split();

                // Connection is accepted. Handle all further in own task
                // TODO -> technically if multiple connect, we would need to send out the messages to ALL connections!!

                let comm_channel = comm_channel.clone();
                let comm_channel_packets = comm_channel_packets.clone();
                let shutdown_marker = shutdown_marker.clone();
                let args = args.clone();
                let state_reader = state_reader.clone();

                tokio::spawn(async move {
                    let comm_channel_read = comm_channel.clone();
                    let mut timing_program_receiver = comm_channel.timing_program_receiver();
                    let comm_channel_packets_read = comm_channel_packets.clone();
                    let mut outbound_packet_receiver = comm_channel_packets.outbound_receiver();
                    let shutdown_marker_read = shutdown_marker.clone();
                    let shutdown_marker_write = shutdown_marker;
                    let args_read = args.clone();
                    let args_write = args;
                    let state_reader_write = state_reader;

                    let read_handler = async move {
                        let mut parser = BufferedParser::new(args_read.clone());
                        let mut buf = [0u8; 65536];
                        loop {
                            if shutdown_marker_read.load(Ordering::SeqCst) {
                                debug!(
                                    "Shutdown marker set, breaking main external -> self transfer"
                                );
                                break;
                            }

                            let n = match time::timeout(
                                Duration::from_millis(
                                    args_read.wait_ms_before_testing_for_shutdown,
                                ),
                                ri.read(&mut buf),
                            )
                            .await
                            {
                                Ok(read_result) => match read_result {
                                    Ok(0) => break,
                                    Ok(n) => n,
                                    Err(e) => return Err(e.to_string()),
                                },
                                Err(_) => {
                                    trace!("No incoming TCP message within timeout interval");
                                    continue;
                                }
                            };

                            // Decoding does NOT log anything. Consider doing so yourself depending on reasonability
                            match parser.feed_bytes(&buf[..n]) {
                                Some(res) => match res {
                                    Err(err) => {
                                        error!("Error when Decoding Inbound Communication: {}", err)
                                    }
                                    Ok(parsed) => {
                                        trace!("Decoded Inbound Communication: {}", parsed);
                                        match comm_channel_read
                                            .take_in_command_from_timing_program(parsed)
                                        {
                                            Ok(()) => (),
                                            Err(e) => return Err(e.to_string()),
                                        }
                                    }
                                },
                                None => trace!(
                                    "Received packet, but does not seeem to be end of communication"
                                ),
                            };

                            // all messages get forwarded unbuffered. Reader needs to take care of it themself
                            if args_read.passthrough_to_display_program {
                                match comm_channel_packets_read.inbound_take_in(buf[..n].into()) {
                                    Ok(_) => (),
                                    Err(e) => return Err(e.to_string()),
                                };
                            }
                        }
                        Ok::<_, String>(())
                    };

                    let write_handler = async move {
                        loop {
                            if shutdown_marker_write.load(Ordering::SeqCst) {
                                debug!(
                                    "Shutdown marker set, breaking main self -> external transfer"
                                );
                                break;
                            }

                            // wait on the tcp stream from passthrough
                            let comm_channel_tcp_outbound_source = async {
                                match outbound_packet_receiver.wait_for_some_data().await {
                                    Ok(Ok(data)) => {
                                        let current_state: ServerState =
                                            state_reader_write.get_server_state().await;

                                        if current_state == ServerState::PassthroughDisplayProgram {
                                            trace!("Display Program sent frame");
                                            return Ok(data);
                                        } else {
                                            return Err(TimeoutOrIoError::SendingBlocked);
                                        }
                                    }
                                    Ok(Err(e)) => return Err(TimeoutOrIoError::ReceiveError(e)),
                                    Err(_) => {
                                        return Err::<PacketData, TimeoutOrIoError>(
                                            TimeoutOrIoError::Timeout,
                                        )
                                    }
                                };
                            };
                            // wait on the back-send-command scheduler
                            let comm_channel_command_outbound_source = async {
                                match timing_program_receiver.wait_for_some_data().await {
                                    Ok(Ok(inst)) => return Ok(inst),
                                    Ok(Err(e)) => return Err(TimeoutOrIoError::ReceiveError(e)),
                                    Err(_) => {
                                        return Err::<InstructionToTimingProgram, TimeoutOrIoError>(
                                            TimeoutOrIoError::Timeout,
                                        )
                                    }
                                };
                            };

                            tokio::select! {
                                r = comm_channel_tcp_outbound_source => {
                                    match r {
                                        Ok(data_to_send) => {
                                            trace!("Proxying TCP back to Timing Program");
                                            if args_write.hexdump_passthrough_communication {
                                                hex_log_bytes(&data_to_send);
                                            }
                                            wi.write_all(&data_to_send)
                                                            .await
                                                            .map_err(|e| e.to_string())?
                                        },
                                        Err(e) => match e {
                                            TimeoutOrIoError::Timeout => {
                                                trace!("No Outgoing TCP or Command to send during timeout interval");
                                                continue;
                                            },
                                            TimeoutOrIoError::SendingBlocked => {
                                                trace!("Data was not proxied back, because we are currently in OUR client mode");
                                                continue;
                                            },
                                            TimeoutOrIoError::ReceiveError(e) => {
                                                return Err(e.to_string());
                                            }
                                        }
                                    }
                                },
                                r = comm_channel_command_outbound_source => {
                                    match r {
                                        Ok(inst) => {
                                            trace!("Sending Bytes for custom command: {}", match inst {
                                                InstructionToTimingProgram::SendFrame(_) => "SendFrame",
                                                InstructionToTimingProgram::SendServerInfo => "ServerInfo",
                                            });
                                            let bytes_to_send = generate_response_bytes(inst);
                                            wi.write_all(&bytes_to_send)
                                                            .await
                                                            .map_err(|e| e.to_string())?
                                        }
                                        Err(e) => match e {
                                            TimeoutOrIoError::Timeout => {
                                                trace!("No Outgoing TCP or Command Connection during timeout interval");
                                                continue;
                                            },
                                            TimeoutOrIoError::SendingBlocked => {
                                                trace!("Data was not proxied back, because we are currently in OUR client mode");
                                                continue;
                                            },
                                            TimeoutOrIoError::ReceiveError(e) => {
                                                return Err(e.to_string());
                                            }
                                        }
                                    }
                                },
                            }
                        }

                        Ok::<_, String>(())
                    };

                    tokio::select! {
                        r = write_handler => r?,
                        r = read_handler => r?,
                    }

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
