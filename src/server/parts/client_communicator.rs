use crate::args::Args;
use crate::hex::hex_log_bytes;
use crate::instructions::{
    ClientCommunicationChannelOutbound, InstructionCommunicationChannel,
    InstructionFromTimingClient, InstructionToTimingClient,
};
use crate::interface::{
    MessageFromClientToServer, MessageFromServerToClient, ServerState, ServerStateMachine,
};
use crate::server::forwarding::{PacketCommunicationChannel, PacketData};
use crate::server::nrbf::{generate_response_bytes, BufferedParser};
use crate::server::xml_serial::{BufferedParserSerial, BufferedParserXML};
use async_channel::RecvError;
use futures::prelude::*;
use std::io::{self, Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{self, sleep};
use tokio_serde::formats::Bincode;
use tokio_serde::Framed;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

/// Start server
pub async fn run_server(args: &Args) -> () {
    let passthrough_address_display_program: SocketAddr = format!(
        "{}:{}",
        args.passthrough_address_display_program, args.passthrough_port_display_program
    )
    .parse()
    .expect("Invalid display_program passthrough address");

    let own_addr_timing: SocketAddr = format!("0.0.0.0:{}", args.listen_port)
        .parse()
        .expect("Invalid listen address");

    if args.passthrough_to_display_program {
        info!(
            "Talking to {} as display program",
            passthrough_address_display_program
        );
    }
    info!(
        "Listening self to the timing program on {}",
        own_addr_timing
    );

    let camera_program_timing_address: SocketAddr = format!(
        "{}:{}",
        args.address_camera_program, args.camera_exchange_timing_port
    )
    .parse()
    .expect("Invalid camera program address for timing");
    let camera_program_data_address: SocketAddr = format!(
        "{}:{}",
        args.address_camera_program, args.camera_exchange_data_port
    )
    .parse()
    .expect("Invalid camera program address for data");
    let camera_program_xml_address: SocketAddr = format!(
        "{}:{}",
        args.address_camera_program, args.camera_exchange_xml_port
    )
    .parse()
    .expect("Invalid camera program address for xml");
    info!(
        "Talking to the camera program on {}, {} and {}",
        camera_program_timing_address, camera_program_data_address, camera_program_xml_address
    );

    let internal_communication_address: SocketAddr = format!(
        "{}:{}",
        args.address_internal_communication, args.internal_communication_port
    )
    .parse()
    .expect("Invalid internal address");

    info!(
        "Talking to {} for internal communication to display client",
        internal_communication_address
    );

    let comm_channel = InstructionCommunicationChannel::new(&args);
    let comm_channel_packets = PacketCommunicationChannel::new(&args);
    let comm_channel_client_outbound = ClientCommunicationChannelOutbound::new(&args);
    let server_state = Arc::new(Mutex::new(ServerStateMachine::new(
        &args,
        comm_channel.clone(),
        comm_channel_client_outbound.clone(),
    )));
    let shutdown_marker = Arc::new(AtomicBool::new(false));

    let tcp_listener_server_instance = tcp_listener_timing_program(
        args.clone(),
        server_state.clone(),
        comm_channel.clone(),
        comm_channel_packets.clone(),
        Arc::clone(&shutdown_marker),
        own_addr_timing,
    );

    let tcp_forwarder_server_display_board_instance = tcp_forwarder_server_display_board(
        args.clone(),
        server_state.clone(),
        comm_channel.clone(),
        comm_channel_packets.clone(),
        Arc::clone(&shutdown_marker),
        passthrough_address_display_program,
    );

    let client_communicator_instance = client_communicator(
        args.clone(),
        server_state,
        comm_channel.clone(),
        comm_channel_client_outbound.clone(),
        Arc::clone(&shutdown_marker),
        internal_communication_address,
    );

    let tcp_client_to_timing_and_data_exchange_instance = tcp_client_to_timing_and_data_exchange(
        args.clone(),
        comm_channel.clone(),
        Arc::clone(&shutdown_marker),
        camera_program_timing_address,
        camera_program_data_address,
        camera_program_xml_address,
    );

    // spawn the async runtimes in parallel
    let client_communicator_task = tokio::spawn(client_communicator_instance);
    let tcp_listener_server_task = tokio::spawn(tcp_listener_server_instance);
    let tcp_forwarder_server_display_board_task =
        tokio::spawn(tcp_forwarder_server_display_board_instance);
    let tcp_client_to_timing_and_data_exchange_task =
        tokio::spawn(tcp_client_to_timing_and_data_exchange_instance);
    let shutdown_task = tokio::spawn(async move {
        // listen for ctrl-c
        tokio::signal::ctrl_c().await?;

        shutdown_marker.store(true, Ordering::SeqCst);

        Ok::<_, Error>(())
    });

    // Wait for all tasks to complete
    match tokio::try_join!(
        tcp_listener_server_task,
        tcp_forwarder_server_display_board_task,
        shutdown_task,
        client_communicator_task,
        tcp_client_to_timing_and_data_exchange_task
    ) {
        Err(_) => error!("Error in at least one listening task"),
        Ok(_) => info!("All listeners closed successfully"),
    };
}

async fn client_communicator(
    args: Args,
    server_state: Arc<Mutex<ServerStateMachine>>,
    comm_channel: InstructionCommunicationChannel,
    comm_channel_client_outbound: ClientCommunicationChannelOutbound,
    shutdown_marker: Arc<AtomicBool>,
    client_addr: SocketAddr,
) -> io::Result<()> {
    let server_state_exchange = server_state.clone();
    let shutdown_marker_exchange = shutdown_marker.clone();
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
                    let comm_channel_client_outbound_write = comm_channel_client_outbound.clone();

                    let write_handler = async move {
                        loop {
                            if shutdown_marker_write.load(Ordering::SeqCst) {
                                debug!(
                                    "Shutdown marker set, breaking main self -> client transfer"
                                );
                                break;
                            }

                            match comm_channel_client_outbound_write
                                .wait_for_message_to_send()
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
                            sleep(Duration::from_millis(1000)).await; // on dev the communication goes into docker, so it connects, then fails. but this spams logs. Slow down retry a bit
                        }
                    }
                }
                Ok(Err(e)) => error!("Client exchange read error: {}", e),
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

async fn tcp_client_to_timing_and_data_exchange(
    args: Args,
    comm_channel: InstructionCommunicationChannel,
    shutdown_marker: Arc<AtomicBool>,
    timing_addr: SocketAddr,
    data_addr: SocketAddr,
    xml_addr: SocketAddr,
) -> io::Result<()> {
    let args_timing = args.clone();
    let shutdown_marker_timing = shutdown_marker.clone();
    let comm_channel_timing = comm_channel.clone();
    let timing_task = async move {
        let mut buf = [0u8; 65536];

        loop {
            if shutdown_marker_timing.load(Ordering::SeqCst) {
                debug!(
                    "Shutdown requested, stopping trying to connect to {}",
                    timing_addr
                );
                break;
            }

            // Wait for new connection with timeout so we can check shutdown flag periodically
            match time::timeout(
                Duration::from_millis(args_timing.wait_ms_before_testing_for_shutdown),
                TcpStream::connect(timing_addr),
            )
            .await
            {
                Ok(Ok(mut timing_stream)) => {
                    debug!("Connected to timing target {}", timing_addr);
                    let mut parser = BufferedParserSerial::new(&args_timing);

                    loop {
                        if shutdown_marker_timing.load(Ordering::SeqCst) {
                            debug!("Shutdown marker set, breaking camera program timing reading");
                            break;
                        }

                        match time::timeout(
                            Duration::from_millis(args_timing.wait_ms_before_testing_for_shutdown),
                            timing_stream.read(&mut buf),
                        )
                        .await
                        {
                            Ok(read_result) => match read_result {
                                Ok(0) => continue,
                                Ok(n) => {
                                    let bytes_from_timing_endpoint = &buf[..n];

                                    match parser.feed_bytes(bytes_from_timing_endpoint) {
                                        Some(Ok(inst)) => {
                                            match comm_channel_timing
                                                .take_in_command_from_camera_program(inst)
                                            {
                                                Ok(()) => (),
                                                Err(e) => {
                                                    return Err(Error::new(
                                                        ErrorKind::Other,
                                                        e.to_string(),
                                                    )); // bad error, internal, can not recover with TCP reconnect
                                                }
                                            }
                                        }
                                        Some(Err(e)) => {
                                            error!(
                                                "Could not parse data from camera program: {}",
                                                e.to_string()
                                            );
                                        }
                                        None => (),
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        "Error in timing channel communication: {}",
                                        e.to_string()
                                    );
                                    break; // try to reconnect
                                }
                            },
                            Err(_) => {
                                trace!("No TCP message on timing channel within timeout interval");
                                continue;
                            }
                        };
                    }
                }
                Ok(Err(e)) => error!("Timing exchange read error: {}", e),
                Err(_) => {
                    // expected on timeout, just loop
                    trace!("No TCP connection to timing exchange could be established within timeout interval");
                }
            }
        }

        Ok::<_, Error>(())
    };

    let args_xml = args.clone();
    let shutdown_marker_xml = shutdown_marker.clone();
    let comm_channel_xml = comm_channel.clone();
    let xml_task = async move {
        let mut buf = [0u8; 65536];

        loop {
            if shutdown_marker_xml.load(Ordering::SeqCst) {
                debug!(
                    "Shutdown requested, stopping trying to connect to {}",
                    xml_addr
                );
                break;
            }

            // Wait for new connection with timeout so we can check shutdown flag periodically
            match time::timeout(
                Duration::from_millis(args_xml.wait_ms_before_testing_for_shutdown),
                TcpStream::connect(xml_addr),
            )
            .await
            {
                Ok(Ok(mut xml_stream)) => {
                    debug!("Connected to xml target {}", xml_addr);
                    let mut parser = BufferedParserXML::new();

                    loop {
                        if shutdown_marker_xml.load(Ordering::SeqCst) {
                            debug!("Shutdown marker set, breaking camera program xml reading");
                            break;
                        }

                        match time::timeout(
                            Duration::from_millis(args_xml.wait_ms_before_testing_for_shutdown),
                            xml_stream.read(&mut buf),
                        )
                        .await
                        {
                            Ok(read_result) => match read_result {
                                Ok(0) => continue,
                                Ok(n) => {
                                    let bytes_from_xml_endpoint = &buf[..n];

                                    match parser.feed_bytes(bytes_from_xml_endpoint) {
                                        Some(Ok(inst)) => {
                                            match comm_channel_xml
                                                .take_in_command_from_camera_program(inst)
                                            {
                                                Ok(()) => (),
                                                Err(e) => {
                                                    return Err(Error::new(
                                                        ErrorKind::Other,
                                                        e.to_string(),
                                                    )); // bad error, internal, can not recover with TCP reconnect
                                                }
                                            }
                                        }
                                        Some(Err(e)) => {
                                            error!(
                                                "Could not parse data from camera program: {}",
                                                e.to_string()
                                            );
                                        }
                                        None => (),
                                    }
                                }
                                Err(e) => {
                                    error!("Error in xml channel communication: {}", e.to_string());
                                    break; // try to reconnect
                                }
                            },
                            Err(_) => {
                                trace!("No TCP message on xml channel within timeout interval");
                                continue;
                            }
                        };
                    }
                }
                Ok(Err(e)) => error!("XML exchange read error: {}", e),
                Err(_) => {
                    // expected on timeout, just loop
                    trace!("No TCP connection to xml exchange could be established within timeout interval");
                }
            }
        }

        Ok::<_, Error>(())
    };

    let args_data = args;
    let shutdown_marker_data = shutdown_marker;
    let comm_channel_data = comm_channel;
    let data_task = async move {
        let mut buf = [0u8; 65536];

        loop {
            if shutdown_marker_data.load(Ordering::SeqCst) {
                debug!(
                    "Shutdown requested, stopping trying to connect to {}",
                    data_addr
                );
                break;
            }

            // Wait for new connection with timeout so we can check shutdown flag periodically
            match time::timeout(
                Duration::from_millis(args_data.wait_ms_before_testing_for_shutdown),
                TcpStream::connect(data_addr),
            )
            .await
            {
                Ok(Ok(mut data_stream)) => {
                    debug!("Connected to data target {}", data_addr);
                    let mut parser = BufferedParserXML::new();

                    loop {
                        if shutdown_marker_data.load(Ordering::SeqCst) {
                            debug!("Shutdown marker set, breaking camera program data reading");
                            break;
                        }

                        match time::timeout(
                            Duration::from_millis(args_data.wait_ms_before_testing_for_shutdown),
                            data_stream.read(&mut buf),
                        )
                        .await
                        {
                            Ok(read_result) => match read_result {
                                Ok(0) => continue,
                                Ok(n) => {
                                    let bytes_from_data_endpoint = &buf[..n];

                                    match parser.feed_bytes(bytes_from_data_endpoint) {
                                        Some(Ok(inst)) => {
                                            match comm_channel_data
                                                .take_in_command_from_camera_program(inst)
                                            {
                                                Ok(()) => (),
                                                Err(e) => {
                                                    return Err(Error::new(
                                                        ErrorKind::Other,
                                                        e.to_string(),
                                                    )); // bad error, internal, can not recover with TCP reconnect
                                                }
                                            }
                                        }
                                        Some(Err(e)) => {
                                            error!(
                                                "Could not parse data from camera program: {}",
                                                e.to_string()
                                            );
                                        }
                                        None => (),
                                    }

                                    continue;
                                }
                                Err(e) => {
                                    error!(
                                        "Error in data channel communication: {}",
                                        e.to_string()
                                    );
                                    break; // try to reconnect
                                }
                            },
                            Err(_) => {
                                trace!("No TCP message on data channel within timeout interval");
                                continue;
                            }
                        };
                    }
                }
                Ok(Err(e)) => error!("Data exchange read error: {}", e),
                Err(_) => {
                    // expected on timeout, just loop
                    trace!("No TCP connection to data exchange could be established within timeout interval");
                }
            }
        }

        Ok::<_, Error>(())
    };

    match tokio::try_join!(timing_task, data_task, xml_task) {
        Err(e) => {
            error!("Error in a camera program listener task");
            return Err(e);
        }
        Ok(_) => info!("All camera program listeners closed successfully"),
    };

    Ok(())
}

enum TimeoutOrIoError {
    Timeout,
    SendingBlocked,
    ReceiveError(RecvError),
}

async fn tcp_listener_timing_program(
    args: Args,
    state: Arc<Mutex<ServerStateMachine>>, // does not require read, but our main reference is in a mutex // TODO could use RWLock
    comm_channel: InstructionCommunicationChannel,
    comm_channel_packets: PacketCommunicationChannel,
    shutdown_marker: Arc<AtomicBool>,
    listen_addr: SocketAddr,
) -> io::Result<()> {
    let listener = TcpListener::bind(listen_addr).await?;
    debug!("TCP listener started on {}", listen_addr);

    loop {
        if shutdown_marker.load(Ordering::SeqCst) {
            debug!("Shutdown requested, stopping listener on {}", listen_addr);
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
                debug!("Accepted connection from {}", client_addr);

                let (mut ri, mut wi) = inbound.into_split();

                // Connection is accepted. Handle all further in own task
                // TODO -> technically if multiple connect, we would need to send out the messages to ALL connections!!

                let comm_channel = comm_channel.clone();
                let comm_channel_packets = comm_channel_packets.clone();
                let shutdown_marker = shutdown_marker.clone();
                let args = args.clone();
                let state = state.clone();

                tokio::spawn(async move {
                    let comm_channel_read = comm_channel.clone();
                    let comm_channel_write = comm_channel;
                    let comm_channel_packets_read = comm_channel_packets.clone();
                    let comm_channel_packets_write = comm_channel_packets;
                    let shutdown_marker_read = shutdown_marker.clone();
                    let shutdown_marker_write = shutdown_marker;
                    let args_read = args.clone();
                    let args_write = args;
                    let state_write = state;

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

                            // Decoding takes care of logging if requested, as there the message is split up to provide more info!!
                            match parser.feed_bytes(&buf[..n]) {
                                Some(res) => match res {
                                    Err(err) => {
                                        error!("Error when Decoding Inbound Communication: {}", err)
                                    }
                                    Ok(parsed) => {
                                        debug!("Decoded Inbound Communication: {}", parsed);
                                        match comm_channel_read
                                            .take_in_command_from_timing_client(parsed)
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
                                match comm_channel_packets_write.outbound_coming_out().await {
                                    Ok(Ok(data)) => {
                                        let current_state: ServerState;
                                        {
                                            let guard = state_write.lock().await;
                                            current_state = guard.state.clone();
                                        }

                                        if current_state == ServerState::PassthroughDisplayBoard {
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
                                match comm_channel_write.wait_for_command_to_send().await {
                                    Ok(Ok(inst)) => return Ok(inst),
                                    Ok(Err(e)) => return Err(TimeoutOrIoError::ReceiveError(e)),
                                    Err(_) => {
                                        return Err::<InstructionToTimingClient, TimeoutOrIoError>(
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
                                                InstructionToTimingClient::SendFrame(_) => "SendFrame",
                                                InstructionToTimingClient::SendServerInfo => "ServerInfo",
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

async fn tcp_forwarder_server_display_board(
    args: Args,
    state: Arc<Mutex<ServerStateMachine>>, // does not require read, but our main reference is in a mutex // TODO could use RWLock
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
            debug!(
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
                debug!("Connected to forwarding target {}", passthrough_target_addr);

                let (mut ro, mut wo) = outbound.into_split();

                // Connection is accepted. only one connection, while one is running

                let comm_channel = comm_channel.clone();
                let comm_channel_packets_read = comm_channel_packets.clone();
                let comm_channel_packets_write = comm_channel_packets.clone();
                let shutdown_marker_read = shutdown_marker.clone();
                let shutdown_marker_write = shutdown_marker.clone();
                let args_read = args.clone();
                let args_write = args.clone();
                let state = state.clone();

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

                                                    let current_state: ServerState;
                                                    {
                                                        let guard = state.lock().await;
                                                        current_state = guard.state.clone();
                                                    }

                                                    if current_state == ServerState::PassthroughDisplayBoard {
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

                                                    let current_state: ServerState;
                                                    {
                                                        let guard = state.lock().await;
                                                        current_state = guard.state.clone();
                                                    }

                                                    match current_state {
                                                        ServerState::PassthroughDisplayBoard => {
                                                            match parsed {
                                                                // THIS IS A BIT OF A HACK -> the "SendFrame" and "SendServerInfo" flow over the "command_from_timing_client" channel, even though they flow in the opposite direction!
                                                                InstructionFromTimingClient::ServerInfo => {
                                                                    match comm_channel.send_out_command(InstructionToTimingClient::SendServerInfo) {
                                                                        Ok(()) => trace!("Detected Packet and queued server-info for rewrite-proxy"),
                                                                        Err(e) => return Err(e.to_string()),
                                                                    }
                                                                }
                                                                InstructionFromTimingClient::SendFrame(frame_data) => {
                                                                    match comm_channel.send_out_command(InstructionToTimingClient::SendFrame(frame_data.clone())) {
                                                                        Ok(()) => trace!("Detected Packet and queued frame for rewrite-proxy"),
                                                                        Err(e) => return Err(e.to_string()),
                                                                    }
                                                                    match comm_channel.take_in_command_from_timing_client(InstructionFromTimingClient::SendFrame(frame_data)) {
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

                        match comm_channel_packets_write.inbound_coming_in().await {
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
