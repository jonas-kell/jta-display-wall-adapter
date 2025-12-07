use crate::args::Args;
use crate::wind::format::WindMessageBroadcast;
use crate::wind::parts::usb_sniffer_parser::decode_single_usb_dump;
use async_broadcast::Sender;
use serialport::SerialPort;
use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

pub fn run_com_port_task(
    args: Args,
    tx_to_tcp: Sender<WindMessageBroadcast>,
    port_path: String,
    shutdown_marker: Arc<AtomicBool>,
) -> std::io::Result<()> {
    trace!("Initializing USB reading Buffer");
    let mut buf = vec![0u8; 1 << 20]; // 1 mb buffer // !! buffer is on the heap, not wasting that much space on the stack -> it crashes windows... from one whimpy mb...
                                      // TODO move other buffers to heap, too (Box or Vec)
    trace!("USB reading Buffer initialized");

    'outer: loop {
        if shutdown_marker.load(Ordering::SeqCst) {
            info!(
                "Shutdown requested, stopping trying to connect to port {}",
                port_path
            );
            break;
        }

        info!("Try open port {} ...", port_path);
        let mut port = match serialport::new(&port_path, 3_000_000)
            .timeout(Duration::from_millis(args.poll_wind_usb_every_nr_ms))
            .open()
        {
            Err(e) => {
                error!("IO error when opening COM port: {}", e.to_string());
                sleep(Duration::from_millis(1000)); // wait shortly before trying to reconnect
                continue;
            }
            Ok(port) => {
                info!("COM port opened!");
                port
            }
        };

        // probably not needed if we correctly set the things below
        // but wait small time for the COM port connection to stabilize
        sleep(std::time::Duration::from_millis(50));

        debug!("Writing terminal ready...");
        // only needed on windows
        match port.write_data_terminal_ready(true) {
            Ok(()) => (),
            Err(e) => {
                error!(
                    "IO error when signaling terminal that we are ready: {}",
                    e.to_string()
                );
                continue;
            }
        };
        // not yet seen what difference this makes, but supposedly a good idea
        match port.write_request_to_send(true) {
            Ok(()) => (),
            Err(e) => {
                error!(
                    "IO error when signaling terminal request to send: {}",
                    e.to_string()
                );
                continue;
            }
        };
        debug!("Terminal ready set!");

        debug!("Starting first capture...");
        match send_data_to_com_port(&mut port, b"s") {
            Err(()) => {
                // retry with fresh connection on error
                continue;
            }
            Ok(()) => {
                debug!("First capture started...");
            }
        };

        loop {
            if shutdown_marker.load(Ordering::SeqCst) {
                info!("Shutdown requested, stopping polling of serial port",);
                break 'outer;
            }

            match port.read(&mut buf) {
                Ok(n) => {
                    // to minimize sniffer downtime, immediately instruct the device to start scanning again
                    match send_data_to_com_port(&mut port, b"s") {
                        Err(()) => {
                            // retry with fresh connection on error
                            break;
                        }
                        Ok(()) => {
                            trace!("Capture restarted");
                        }
                    };

                    debug!("Decoding data from sniffer");
                    for result in decode_single_usb_dump(&buf[..n]) {
                        match result {
                            Ok(msg) => {
                                // not blocking, as we have set overflow_mode. if old message is returned from this, we just throw it away
                                match tx_to_tcp.try_broadcast(msg) {
                                    Ok(_) => {
                                        trace!("Thrown away old message in internal comm channel");
                                    }
                                    Err(e) => {
                                        warn!(
                                            "Internal channel not open, no active receivers: {}",
                                            e.to_string()
                                        );
                                        continue;
                                    }
                                }
                                trace!("Passed new wind command into internal broadcast channel");
                            }
                            Err(e) => {
                                error!("Error when decoding a part of the USB sniffer communication: {}", e.to_string());
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                    trace!("Read timeout hit");
                    match send_data_to_com_port(&mut port, b"p") {
                        Err(()) => {
                            // retry with fresh connection on error
                            break;
                        }
                        Ok(()) => {
                            trace!("Instructed client device to flushprint buffer");
                        }
                    };
                }
                Err(e) => {
                    error!("Unknown read error on port: {}", e.to_string());
                    // try to recover by fresh reconnection
                    break;
                }
            }
        }
    }

    Ok(())
}

fn send_data_to_com_port(port: &mut Box<dyn SerialPort>, data: &[u8]) -> Result<(), ()> {
    trace!("Writing data to COM port: {:?}", data);

    match port.write_all(data) {
        Ok(()) => {
            trace!("Successfully written data to COM port");
            Ok(())
        }
        Err(e) => {
            error!("Error when writing data to COM port: {}", e.to_string());
            Err(())
        }
    }
}
