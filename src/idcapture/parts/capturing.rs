use pcap::Device;
use std::{net::IpAddr, str::FromStr, time::Duration};
use tokio::time::sleep;

pub async fn capture(dev: Device, filter: Option<(IpAddr, IpAddr, u16)>) {
    loop {
        // let dev = dev.clone();
        let dev = Device::lookup().unwrap().unwrap(); // works
        let dev_name = dev.name.clone();

        info!("Starting packet capture on device: {}", dev_name);

        let mut cap = match dev.open() {
            Ok(cap) => cap,
            Err(e) => {
                error!(
                    "Could not open capture on device {}: {}",
                    dev_name,
                    e.to_string()
                );
                continue;
            }
        };

        debug!("Got an open capture");

        // if let Some((source_ip, target_ip, port)) = filter {
        //     match cap.filter(
        //         &format!(
        //             "tcp and src host {} and dst host {} and dst port {}",
        //             source_ip, target_ip, port
        //         ),
        //         true,
        //     ) {
        //         Ok(()) => {}
        //         Err(e) => {
        //             error!("Error while setting filter: {}", e.to_string());
        //             continue;
        //         }
        //     };
        // }

        // debug!("Set filter on capture if applicable");

        let listening_task = tokio::task::spawn_blocking(move || {
            debug!("Start listening on the capture");

            while let Ok(packet) = cap.next_packet() {
                trace!("Received packet! {:?}", packet);
            }
        });
        let _ = listening_task.await; // TODO

        warn!("Capturing device went away - retrying");
        sleep(Duration::from_millis(1000)).await;
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
