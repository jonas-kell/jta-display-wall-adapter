use crate::args::Args;
use crate::hex::hex_log_bytes;
use crate::instructions::{InstructionFromTimingProgram, InstructionToTimingProgram};
use crate::interface::{ServerState, ServerStateMachineServerStateReader};
use crate::server::comm_channel::{InstructionCommunicationChannel, PacketCommunicationChannel};
use crate::server::nrbf::BufferedParser;
use std::io;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time;

pub async fn tcp_forwarder_display_program(
    args: Args,
    state_reader: ServerStateMachineServerStateReader,
    comm_channel: InstructionCommunicationChannel,
    comm_channel_packets: PacketCommunicationChannel,
    shutdown_marker: Arc<AtomicBool>,
    passthrough_target_addr: SocketAddr,
) -> io::Result<()> {
    // never send stuff out here -> we can just die
    if !args.passthrough_to_display_program {
        return Ok(());
    };

    loop {
        if shutdown_marker.load(Ordering::SeqCst) {
            info!(
                "Shutdown requested, stopping trying to connect to {}",
                passthrough_target_addr
            );
            break;
        }

        // Wait for new connection with timeout so we can check shutdown flag periodically
        match time::timeout(
            Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
            TcpStream::connect(passthrough_target_addr),
        )
        .await
        {
            Ok(Ok(outbound)) => {
                info!("Connected to forwarding target {}", passthrough_target_addr);

                let (mut ro, mut wo) = outbound.into_split();

                // Connection is accepted. only one connection, while one is running

                let comm_channel = comm_channel.clone();
                let comm_channel_packets_read = comm_channel_packets.clone();
                let mut inbound_packet_receiver = comm_channel_packets.inbound_receiver();
                let shutdown_marker_read = shutdown_marker.clone();
                let shutdown_marker_write = shutdown_marker.clone();
                let args_read = args.clone();
                let args_write = args.clone();
                let state_reader = state_reader.clone();

                let read_handler = async move {
                    let mut buf = [0u8; 65536];
                    let mut parser = BufferedParser::new(args_read.clone());

                    loop {
                        if shutdown_marker_read.load(Ordering::SeqCst) {
                            debug!("Shutdown marker set, breaking proxy external -> self transfer");
                            break;
                        }

                        match time::timeout(
                            Duration::from_millis(args_read.wait_ms_before_testing_for_shutdown),
                            ro.read(&mut buf),
                        )
                        .await
                        {
                            Ok(read_result) => {
                                match read_result {
                                    Ok(0) => continue,
                                    Ok(n) => {
                                        // Decoding takes care of logging if requested, as there the message is split up to provide more info!!
                                        match parser.feed_bytes_return_owned_on_fail(&buf[..n]) {
                                            Some(res) => match res {
                                                Err((err, data_that_could_not_be_parsed)) => {
                                                    error!("Error when Decoding Outbound Communication: {}", err);

                                                    let current_state: ServerState = state_reader.get_server_state().await;

                                                    if current_state == ServerState::PassthroughDisplayProgram {
                                                        // proxy just like that if not successfully parsed
                                                        match comm_channel_packets_read
                                                            .outbound_take_in(data_that_could_not_be_parsed)
                                                        {
                                                            Ok(()) => trace!(
                                                                "Sending onwards a Packet that could not be decoded (proxy back)"
                                                            ),
                                                            Err(e) => return Err(e.to_string()),
                                                        };
                                                    }
                                                }
                                                Ok(parsed) => {
                                                    trace!(
                                                        "Decoded Outbound Communication: {}",
                                                        parsed
                                                    );

                                                    let current_state: ServerState = state_reader.get_server_state().await;

                                                    match current_state {
                                                        ServerState::PassthroughDisplayProgram => {
                                                            match parsed {
                                                                // THIS IS A BIT OF A HACK -> the "SendFrame" and "SendServerInfo" flow over the "command_from_timing_program" channel, even though they flow in the opposite direction!
                                                                InstructionFromTimingProgram::ServerInfo => {
                                                                    match comm_channel.send_out_command_to_timing_program(InstructionToTimingProgram::SendServerInfo) {
                                                                        Ok(()) => trace!("Detected Packet and queued server-info for rewrite-proxy"),
                                                                        Err(e) => return Err(e.to_string()),
                                                                    }
                                                                }
                                                                InstructionFromTimingProgram::SendFrame(frame_data) => {
                                                                    match comm_channel.send_out_command_to_timing_program(InstructionToTimingProgram::SendFrame(frame_data.clone())) {
                                                                        Ok(()) => trace!("Detected Packet and queued frame for rewrite-proxy"),
                                                                        Err(e) => return Err(e.to_string()),
                                                                    }
                                                                    match comm_channel.take_in_command_from_timing_program(InstructionFromTimingProgram::SendFrame(frame_data)) {
                                                                        Ok(()) => trace!("Detected Packet and queued frame into communication interface"),
                                                                        Err(e) => return Err(e.to_string()),
                                                                    }
                                                                },
                                                                comm => {
                                                                    error!("Unexpected: got a command OUTBOUND that should not happen: {}", comm);
                                                                }
                                                            }
                                                        }
                                                        ServerState::PassthroughClient => {
                                                            trace!("Outwards flowing data blocked");
                                                        }
                                                    }
                                                }
                                            },
                                            None => trace!(
                                                "Received packet, but does not seeem to be end of communication"
                                            ),
                                        };
                                    }
                                    Err(e) => return Err(e.to_string()),
                                }
                            }
                            Err(_) => trace!("No TCP to proxy back during timeout interval"),
                        }
                    }

                    Ok::<_, String>(())
                };

                let write_handler = async move {
                    loop {
                        if shutdown_marker_write.load(Ordering::SeqCst) {
                            debug!("Shutdown marker set, breaking proxy self -> external transfer");
                            break;
                        }

                        match inbound_packet_receiver.wait_for_some_data().await {
                            Ok(Ok(data)) => {
                                trace!("Proxying through packet from timing program to display program unaltered");
                                if args_write.hexdump_passthrough_communication {
                                    hex_log_bytes(&data);
                                }

                                wo.write_all(&data).await.map_err(|e| e.to_string())?;
                            }
                            Ok(Err(e)) => return Err(e.to_string()),
                            Err(_) => {
                                trace!("Nothing to send out to proxy during timeout interval");
                                continue;
                            }
                        };
                    }

                    Ok::<_, String>(())
                };

                match tokio::try_join!(read_handler, write_handler) {
                    Err(e) => error!("Error in proxy read or write: {}", e),
                    Ok(_) => info!("Proxy connection closed successfully"),
                };
            }
            Ok(Err(e)) => error!("Proxy error: {}", e),
            Err(_) => {
                // expected on timeout, just loop
                trace!("No TCP connection to proxy anything could be established within timeout interval");
            }
        }
    }

    Ok(())
}
