use crate::args::Args;
use crate::instructions::{InstructionCommunicationChannel, InstructionToTimingClient};
use crate::nrbf::{decode_single_nrbf, generate_response_bytes, hex_log_bytes};
use async_channel::RecvError;
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
use tokio::time::{self, sleep};

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

    let comm_channel = InstructionCommunicationChannel::new(&args);

    let shutdown_marker = Arc::new(AtomicBool::new(false));

    let tcp_listener_server_instance = tcp_listener_server(
        args.clone(),
        comm_channel.clone(),
        Arc::clone(&shutdown_marker),
        own_addr_timing,
        passthrough_address_display_program,
    );

    let client_communicator_instance = client_communicator(
        args.clone(),
        comm_channel.clone(),
        Arc::clone(&shutdown_marker),
    );

    // spawn the async runtimes in parallel
    let client_communicator_task = tokio::spawn(client_communicator_instance);
    let tcp_listener_server_task = tokio::spawn(tcp_listener_server_instance);
    let shutdown_task = tokio::spawn(async move {
        // listen for ctrl-c
        tokio::signal::ctrl_c().await?;

        shutdown_marker.store(true, Ordering::SeqCst);

        Ok::<_, Error>(())
    });

    // Wait for all tasks to complete
    match tokio::try_join!(
        tcp_listener_server_task,
        shutdown_task,
        client_communicator_task
    ) {
        Err(_) => error!("Error in at least one listening task"),
        Ok(_) => info!("All listeners closed successfully"),
    };
}

async fn client_communicator(
    args: Args,
    comm_channel: InstructionCommunicationChannel,
    shutdown_marker: Arc<AtomicBool>,
) -> io::Result<()> {
    let _ = args;

    let comm_channel_a = comm_channel.clone();
    let print_commands_task = tokio::spawn(async move {
        loop {
            if shutdown_marker.load(Ordering::SeqCst) {
                debug!("Shutdown requested, stopping client communicator");
                break;
            }

            match comm_channel_a.wait_for_incomming_command().await {
                Ok(command_res) => match command_res {
                    Ok(comm) => {
                        info!("Command received!!: {:?}", comm);
                    }
                    Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
                },
                Err(_) => {
                    trace!("No incoming command to report");
                    continue;
                }
            };
        }
        Ok::<_, Error>(())
    });

    let send_test_commands_task = tokio::spawn(async move {
        sleep(Duration::from_secs(2)).await;
        debug!("Sending Frame...");
        match comm_channel
            .send_out_command(InstructionToTimingClient::SendBeforeFrameSetupInstruction)
            .await
        {
            Ok(()) => trace!("Command queued"),
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        };
        match comm_channel
            .send_out_command(InstructionToTimingClient::SendFrame)
            .await
        {
            Ok(()) => trace!("Command queued"),
            Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
        };

        Ok::<_, Error>(())
    });

    let (a, b) = tokio::try_join!(print_commands_task, send_test_commands_task)?;
    a?;
    b?;

    Ok(())
}

async fn tcp_listener_server(
    args: Args,
    comm_channel: InstructionCommunicationChannel,
    shutdown_marker: Arc<AtomicBool>,
    listen_addr: SocketAddr,
    passthrough_target_addr: SocketAddr,
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

                let passthrough_target_addr = passthrough_target_addr.clone();
                let shutdown_marker = shutdown_marker.clone();
                let comm_channel = comm_channel.clone();
                let args = args.clone();

                if args.passthrough_to_display_program {
                    tokio::spawn(async move {
                        // TODO this could be made to timeout (?)
                        match TcpStream::connect(passthrough_target_addr).await {
                            Ok(outbound) => {
                                debug!("Connected to target {}", passthrough_target_addr);
                                if let Err(e) = transfer_optional_bidirectional(
                                    args,
                                    inbound,
                                    Some(outbound),
                                    shutdown_marker.clone(),
                                    comm_channel,
                                )
                                .await
                                {
                                    error!(
                                        "Error during passthrough transfer between {} and {}: {}",
                                        client_addr, passthrough_target_addr, e
                                    );
                                } else {
                                    debug!(
                                        "Closed passthrough connection between {} and {}",
                                        client_addr, passthrough_target_addr
                                    );
                                }
                            }
                            Err(e) => error!(
                                "Failed to connect to target {} for passthrough: {}",
                                passthrough_target_addr, e
                            ),
                        }
                    });
                } else {
                    tokio::spawn(async move {
                        if let Err(e) = transfer_optional_bidirectional(
                            args,
                            inbound,
                            None,
                            shutdown_marker.clone(),
                            comm_channel,
                        )
                        .await
                        {
                            error!("Error during communication with {}: {}", client_addr, e);
                        } else {
                            debug!("Closed connection to {}", client_addr);
                        }
                    });
                }
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

enum TimeoutOrIoError {
    Timeout,
    IoError(io::Error),
    ReceiveError(RecvError),
}

async fn transfer_optional_bidirectional(
    args: Args,
    inbound: TcpStream,
    outbound: Option<TcpStream>,
    shutdown_marker: Arc<AtomicBool>,
    comm_channel: InstructionCommunicationChannel,
) -> Result<(), String> {
    let (mut ri, mut wi) = inbound.into_split();
    let (mut ro, mut wo) = match outbound.map(|o| o.into_split()) {
        Some((ro, wo)) => (Some(ro), Some(wo)),
        None => (None, None),
    };

    let client_to_server = async {
        let mut buf = [0u8; 65536];
        loop {
            if shutdown_marker.load(Ordering::SeqCst) {
                debug!("Shutdown marker set, breaking client -> server transfer");
                break;
            }

            let n = match time::timeout(
                Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
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
            match decode_single_nrbf(&args, &buf[..n]) {
                Err(err) => error!("Error when Decoding Inbound Communication: {}", err),
                Ok(parsed) => {
                    debug!("Decoded Inbound Communication: {:?}", parsed);
                    match comm_channel.take_in_command(parsed).await {
                        Ok(()) => (),
                        Err(e) => return Err(e.to_string()),
                    }
                }
            };

            // if there is something we need to passthrough (otherwise it would be None)
            if let Some(wo) = &mut wo {
                wo.write_all(&buf[..n]).await.map_err(|e| e.to_string())?;
            }
        }
        Ok::<_, String>(())
    };

    let server_to_client = async {
        let mut buf = [0u8; 65536];
        loop {
            if shutdown_marker.load(Ordering::SeqCst) {
                debug!("Shutdown marker set, breaking server -> client transfer");
                break;
            }

            // wait on the tcp stream from passthrough
            let tcp_inbound_source = async {
                // if there is something we need to passthrough (otherwise it would be None)
                if let Some(ro) = &mut ro {
                    match time::timeout(
                        Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
                        ro.read(&mut buf),
                    )
                    .await
                    {
                        Ok(read_result) => match read_result {
                            Ok(0) => Ok(None),
                            Ok(n) => Ok(Some(n)),
                            Err(e) => Err(TimeoutOrIoError::IoError(e)),
                        },
                        Err(_) => Err(TimeoutOrIoError::Timeout),
                    }
                } else {
                    time::sleep(Duration::from_millis(
                        args.wait_ms_before_testing_for_shutdown,
                    ))
                    .await;
                    Err::<Option<usize>, TimeoutOrIoError>(TimeoutOrIoError::Timeout)
                }
            };
            // wait on the back-send-command scheduler
            let comm_channel_inbound_source = async {
                match comm_channel.wait_for_command_to_send().await {
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
                r = tcp_inbound_source => {
                    match r {
                        Ok(Some(n)) => {
                            trace!("Proxying TCP back to Timing Program");
                            if args.hexdump_passthrough_communication {
                                hex_log_bytes(&buf[..n]);
                            }
                            wi.write_all(&buf[..n])
                                            .await
                                            .map_err(|e| e.to_string())?
                        },
                        Ok(None) => continue,
                        Err(e) => match e {
                            TimeoutOrIoError::Timeout => {
                                trace!("No Outgoing TCP or Command Connection during timeout interval");
                                continue;
                            },
                            TimeoutOrIoError::IoError(e) => {
                                return Err(e.to_string());
                            },
                            TimeoutOrIoError::ReceiveError(e) => {
                                return Err(e.to_string());
                            }
                        }
                    }
                },
                r = comm_channel_inbound_source => {
                    match r {
                        Ok(inst) => {
                            trace!("Sending Bytes for custom command: {:?}", inst);
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
                            TimeoutOrIoError::IoError(e) => {
                                return Err(e.to_string());
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
        r = client_to_server => r?,
        r = server_to_client => r?,
    }

    Ok(())
}
