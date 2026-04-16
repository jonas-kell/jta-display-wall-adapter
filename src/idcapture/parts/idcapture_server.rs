use crate::{
    args::{Args, MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS},
    idcapture::{
        format::IDCaptureMessage,
        parts::{
            capturing::{capture, find_device_to_capture},
            tcp::run_network_task,
        },
    },
};
use std::io::Error;
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub async fn run_idcapture_server(args: &Args) -> () {
    info!("Starting idcapture server.");

    let listen_addr: SocketAddr = format!("0.0.0.0:{}", args.idcapture_exchange_port)
        .parse()
        .expect("Invalid idcapture exchange address");

    info!("TCP server will be bound to {}", listen_addr);

    let (mut tx_out_to_tcp, rx_in_from_capture_task) =
        async_broadcast::broadcast::<IDCaptureMessage>(MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS);
    tx_out_to_tcp.set_overflow(true); // do not care, if messages get lost. They there wouuld have been no tcp client to receive them anyway, it seems
    let rx_in_from_capture_task = rx_in_from_capture_task.deactivate();

    // !! does not work with "any" - good to use "lo"
    let dev = match find_device_to_capture(args.idcapture_interface_filter.clone()) {
        Ok(dev) => dev,
        Err(e) => {
            error!("Could not start idcapture server: {}", e);
            return;
        }
    };

    let filter = match &args.idcapture_target_address {
        None => None,
        Some(address) => match IpAddr::from_str(address) {
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

    info!(
        "Capture device found. Starting packet listener on dev {} and TCP server on {}",
        dev.name, listen_addr
    );

    let shutdown_marker = Arc::new(AtomicBool::new(false));

    let network_task = tokio::spawn(run_network_task(
        args.clone(),
        listen_addr,
        rx_in_from_capture_task,
        Arc::clone(&shutdown_marker),
    ));
    let capture_task_shutdown_marker = shutdown_marker.clone();
    let capture_task_args = args.clone();
    let capture_task = tokio::task::spawn_blocking(move || {
        capture(
            capture_task_args,
            dev,
            filter,
            tx_out_to_tcp,
            capture_task_shutdown_marker,
        )
    });
    let shutdown_marker_sdt = Arc::clone(&shutdown_marker);
    let shutdown_task = tokio::spawn(async move {
        // listen for ctrl-c
        tokio::signal::ctrl_c().await?;

        shutdown_marker_sdt.store(true, Ordering::SeqCst);

        Ok::<_, Error>(())
    });

    match tokio::try_join!(capture_task, network_task, shutdown_task) {
        Err(_) => error!("Error in at least one idcapture server task"),
        Ok(_) => info!("All idcapture server tasks closed successfully"),
    };
}
