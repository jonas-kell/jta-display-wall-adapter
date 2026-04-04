use crate::{
    args::Args,
    idcapture::parts::capturing::{capture, find_device_to_capture},
};
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

    let filter = match args.idcapture_target_address {
        None => None,
        Some(address) => match IpAddr::from_str(&address) {
            Ok(add) => Some((add, args.idcapture_target_port)),
            Err(e) => {
                error!(
                    "idcapture_target_address could not be parsed: {}",
                    e.to_string()
                );
                None
            }
        },
    };

    capture(dev, filter).await;
}
