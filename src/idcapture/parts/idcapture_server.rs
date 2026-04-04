use crate::{
    args::Args,
    idcapture::parts::capturing::{capture, find_device_to_capture},
};
use pcap::Device;
use std::{net::IpAddr, str::FromStr};

pub async fn run_idcapture_server(args: &Args) -> () {
    info!("Starting idcapture server.");

    // !! does not work with "any" - good to use "lo"
    let dev = match find_device_to_capture(args.idcapture_interface_filter.clone()) {
        Ok(dev) => dev,
        Err(e) => {
            error!("Could not start idcapture server: {}", e);
            return;
        }
    };

    let filter = Some((
        IpAddr::from_str("127.0.0.1").unwrap(),
        IpAddr::from_str("127.0.0.1").unwrap(),
        args.idcapture_port,
    ));

    capture(dev, filter).await;
}
