use etherparse::{NetSlice, SlicedPacket, TransportHeader, TransportSlice};
use pcap::{Capture, Device, Error};
use std::{net::IpAddr, str::FromStr, time::Duration};
use tokio::time::sleep;

use crate::hex::get_hex_repr;

pub async fn capture(dev: Device, filter: Option<(IpAddr, IpAddr, u16)>) {
    loop {
        let dev = dev.clone();
        let dev_name = dev.name.clone();

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

        if let Some((source_ip, target_ip, port)) = filter {
            match cap.filter(
                &format!(
                    "tcp and src host {} and dst host {} and dst port {}",
                    source_ip, target_ip, port
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

        let listening_task = tokio::task::spawn_blocking(move || {
            debug!("Start listening on the capture");

            loop {
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
                                        trace!("Payload length: {}", payload.len());
                                        trace!("Payload: {}", get_hex_repr(payload));
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
        let _ = listening_task.await; // TODO better handling

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
                "Provided filter could not be parsed as an IP address: {}",
                e
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
