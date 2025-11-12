use crate::args::Args;
use crate::instructions::InstructionCommunicationChannel;
use crate::server::xml_serial::{BufferedParserSerial, BufferedParserXML};
use std::io::{self, Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::time::{self, sleep};

pub async fn tcp_client_camera_program(
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
                                                "Could not parse data from camera program (timing target): {}",
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
                Ok(Err(e)) => {
                    error!("Timing exchange error: {}", e);
                    sleep(Duration::from_millis(1000)).await; // on missing target the communication sometimes connects with "Error - connection refused" -> immediately fails. But this spams logs. Slow down retry a bit
                }
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
                                                "Could not parse data from camera program (xml target): {}",
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
                Ok(Err(e)) => {
                    error!("XML exchange error: {}", e);
                    sleep(Duration::from_millis(1000)).await; // on missing target the communication sometimes connects with "Error - connection refused" -> immediately fails. But this spams logs. Slow down retry a bit
                }
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
                                                "Could not parse data from camera program (data target): {}",
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
                Ok(Err(e)) => {
                    error!("Data exchange error: {}", e);
                    sleep(Duration::from_millis(1000)).await; // on missing target the communication sometimes connects with "Error - connection refused" -> immediately fails. But this spams logs. Slow down retry a bit
                }
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
