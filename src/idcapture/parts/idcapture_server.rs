use crate::{
    args::Args,
    idcapture::parts::capturing::{capture, find_device_to_capture},
};
use pcap::Device;
use std::{net::IpAddr, str::FromStr};

pub async fn run_idcapture_server(args: &Args) -> () {
    info!("Starting idcapture server.");

    let dev = match find_device_to_capture(Some("lo".into())) {
        Ok(dev) => dev,
        Err(e) => {
            error!("Could not start idcapture server: {}", e);
            return;
        }
    };

    capture(
        dev,
        Some((
            IpAddr::from_str("127.0.0.1").unwrap(),
            IpAddr::from_str("127.0.0.1").unwrap(),
            9797,
        )),
    )
    .await;
}
