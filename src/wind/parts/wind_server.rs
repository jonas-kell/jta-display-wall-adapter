use crate::args::Args;
use crate::wind::format::WindMessageBroadcast;
use crate::wind::parts::comport_adapter::run_com_port_task;
use crate::wind::parts::tcp::run_network_task;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub async fn run_wind_server(args: &Args) -> () {
    info!("Starting wind server.");

    let listen_addr: SocketAddr = format!("0.0.0.0:{}", args.wind_exchange_port)
        .parse()
        .expect("Invalid wind exchange address");
    info!("TCP server will be bound to {}", listen_addr);

    // wind server does not need to store many messages. Will just trash them earlier
    let (mut tx_out_to_tcp, rx_in_from_com_port) =
        async_broadcast::broadcast::<WindMessageBroadcast>(3); // one data frame, one mesaurement frame and one space at least (more should not be read in one iteration and is also not interesting for us)
    tx_out_to_tcp.set_overflow(true); // do not care, if messages get lost. They there wouuld have been no tcp client to receive them anyway, it seems

    // linux: "/dev/serial/by-id/usb-Alex_Taradov_USB_Sniffer_Lite__RP2040__7A6B5C58-if00";
    let port_path = args.wind_usb_sniffer_address.clone();
    // TODO Windows expects strange \\\\.\\ prefix for higher number COM ports (from 10 on).

    info!("Will connect to USB sniffer at {}", port_path);

    let shutdown_marker = Arc::new(AtomicBool::new(false));

    let network_task = tokio::spawn(run_network_task(
        args.clone(),
        listen_addr,
        rx_in_from_com_port,
        Arc::clone(&shutdown_marker),
    ));
    let com_task_args = args.clone();
    let com_task_shutdown_marker = shutdown_marker.clone();
    let com_port_task = tokio::task::spawn_blocking(move || {
        // this is not any async task, but blocking because the serialport lib is blocking
        run_com_port_task(
            com_task_args,
            tx_out_to_tcp,
            port_path,
            com_task_shutdown_marker,
        )
    });
    let shutdown_marker_sdt = Arc::clone(&shutdown_marker);
    let shutdown_task = tokio::spawn(async move {
        // listen for ctrl-c
        tokio::signal::ctrl_c().await?;

        shutdown_marker_sdt.store(true, Ordering::SeqCst);

        Ok::<_, Error>(())
    });

    match tokio::try_join!(com_port_task, network_task, shutdown_task) {
        Err(_) => error!("Error in at least one wind server task"),
        Ok(_) => info!("All wind server tasks closed successfully"),
    };
}
