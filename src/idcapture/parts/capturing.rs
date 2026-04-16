use crate::{idcapture::format::IDCaptureMessage, nrbf::BufferedParser, Args};
use async_broadcast::{Sender, TrySendError};
use etherparse::{NetSlice, SlicedPacket, TransportSlice};
use pcap::{Capture, Device, Error};
use std::{
    net::IpAddr,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::time::sleep;

pub async fn capture(
    args: Args,
    dev: Device,
    filter: Option<(IpAddr, u16)>,
    tx_to_tcp: Sender<IDCaptureMessage>,
    shutdown_marker: Arc<AtomicBool>,
) {
    loop {
        let dev = dev.clone();
        let dev_name = dev.name.clone();

        if shutdown_marker.load(Ordering::SeqCst) {
            info!(
                "Shutdown requested, stopping packet capture on device {}",
                dev_name
            );
            break;
        }

        info!("Starting packet capture on device: {}", dev_name);

        let cap_inact = match Capture::from_device(dev) {
            Ok(cap) => cap,
            Err(e) => {
                error!(
                    "Could not create inactive capture on device {}: {}",
                    dev_name,
                    e.to_string()
                );
                sleep(Duration::from_millis(1000)).await;
                continue;
            }
        };

        let mut cap = match cap_inact.immediate_mode(true).timeout(0).open() {
            Ok(cap) => cap,
            Err(e) => {
                error!(
                    "Could not activate capture on device {}: {}",
                    dev_name,
                    e.to_string()
                );
                sleep(Duration::from_millis(1000)).await;
                continue;
            }
        };

        debug!("Got an open capture");

        if let Some((target_ip, port)) = filter {
            match cap.filter(
                &format!(
                    // "tcp and src host {} and dst host {} and dst port {}",
                    "tcp and dst host {} and dst port {}",
                    target_ip, port
                ),
                true,
            ) {
                Ok(()) => {}
                Err(e) => {
                    error!("Error while setting filter: {}", e.to_string());
                    sleep(Duration::from_millis(1000)).await;
                    continue;
                }
            };

            debug!("Set filter on capture if applicable");
        }

        let shutdown_marker = shutdown_marker.clone();
        let tx_to_tcp = tx_to_tcp.clone();
        let args = args.clone();
        let listening_task = tokio::task::spawn_blocking(move || {
            debug!("Start listening on the capture");

            let mut parser = BufferedParser::new(args);

            loop {
                if shutdown_marker.load(Ordering::SeqCst) {
                    info!(
                        "Shutdown requested, stopping packet capture on device {}",
                        dev_name
                    );
                    break;
                }

                match cap.next_packet() {
                    Ok(packet) => {
                        trace!(
                            "Received packet with len {} on interface {}",
                            packet.len(),
                            dev_name,
                        );

                        if let Ok(ether_pack) = SlicedPacket::from_ethernet(&packet.data) {
                            if let Some(net) = ether_pack.net {
                                let (ip_from, ip_to) = match net {
                                    NetSlice::Ipv4(slice) => {
                                        let src = slice.header().source_addr();
                                        let dst = slice.header().destination_addr();
                                        (src.to_string(), dst.to_string())
                                    }
                                    NetSlice::Ipv6(slice) => {
                                        let src = slice.header().source_addr();
                                        let dst = slice.header().destination_addr();
                                        (src.to_string(), dst.to_string())
                                    }
                                    _ => ("unknown".into(), "unknown".into()),
                                };
                                if let Some(tcp) = ether_pack.transport {
                                    if let TransportSlice::Tcp(tcp_slice) = tcp {
                                        trace!(
                                            "{}:{} -> {}:{}",
                                            ip_from,
                                            tcp_slice.source_port(),
                                            ip_to,
                                            tcp_slice.destination_port()
                                        );

                                        let payload = tcp_slice.payload();
                                        // no payload tcp messages are not relevant for us...
                                        if payload.len() > 0 {
                                            // Decoding takes care of logging if requested, as there the message is split up to provide more info!!

                                            match parser.feed_bytes(payload) {
                                                Some(res) => match res {
                                                    Err(err) => {
                                                        error!("Error when Decoding Outbound Communication: {}", err);
                                                    }
                                                    Ok(parsed) => {
                                                        match parsed.into_idcapture_instruction() {
                                                            Ok(msg) => {
                                                                // not blocking, as we have set overflow_mode. if old message is returned from this, we just throw it away
                                                                match tx_to_tcp.try_broadcast(msg) {
                                                                    Ok(Some(_)) => {
                                                                        trace!("Thrown away old message in internal comm channel");
                                                                    }
                                                                    Ok(None) => (),
                                                                    Err(
                                                                        TrySendError::Inactive(_),
                                                                    ) => {
                                                                        warn!("Internal channel not open, no active receivers");
                                                                        continue;
                                                                    }
                                                                    Err(TrySendError::Full(_)) => {
                                                                        error!("Receivers are there, but internal channel full. This should not happen!");
                                                                        continue;
                                                                    }
                                                                    Err(TrySendError::Closed(
                                                                        _,
                                                                    )) => {
                                                                        error!("Internal comm channel went away unexpectedly");
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                            Err(e_inst) => {
                                                                error!("Should never get this instruction from this channel. {} Something is mis-connected", e_inst);
                                                            }
                                                        }
                                                    }
                                                }
                                                None => trace!(
                                                    "Received packet, but does not seeem to be end of communication"
                                                ),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => match e {
                        Error::TimeoutExpired => {}
                        e => {
                            error!("Error: {}", e.to_string());
                            break; // This was never reached in testing. Think about if other error types could also be non-fatal
                        }
                    },
                }
            }
        });
        match listening_task.await {
            Ok(_) => (),
            Err(e) => {
                error!("Error on waiting for the listening task: {}", e.to_string())
            }
        }

        warn!("Capturing device went away - reconnecting");
    }
}

pub fn find_device_to_capture(filter: Option<String>) -> Result<Device, String> {
    let filter =
        match filter {
            Some(filter) => filter,
            None => {
                debug!("No filter for device applied. Try using libary standard");

                let dev_opt = Device::lookup().map_err(|e| {
                    format!(
                        "Default capture device could not be determined: {}",
                        e.to_string()
                    )
                })?;
                match dev_opt {
                    Some(dev) => return Ok(dev),
                    None => return Err(String::from(
                        "No errors happened, but no default device to capture could be determined",
                    )),
                }
            }
        };

    let ip_filter = match IpAddr::from_str(&filter) {
        Ok(ip) => Some(ip),
        Err(e) => {
            warn!(
                "Provided filter  >>{}<< could not be parsed as an IP address: {}",
                filter, e
            );
            None
        }
    };
    let name_filter = match ip_filter {
        Some(_) => None,
        None => Some(filter),
    };

    let list = Device::list().map_err(|e| {
        format!(
            "Network capture devices could not be enumerated: {}",
            e.to_string()
        )
    })?;

    for dev in list.clone() {
        if Some(&dev.name) == name_filter.as_ref() {
            return Ok(dev);
        }
        for addr in &dev.addresses {
            if Some(addr.addr) == ip_filter {
                return Ok(dev);
            }
        }
    }

    info!("Devices that were found for capture: \n{:#?}", list);
    return Err("No capture devices matched the filter requiremenets".into());
}
