use crate::args::{Args, MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS};
use crate::client::parts::app::run_display_task;
use crate::client::parts::tcp::run_network_task;
use crate::interface::{MessageFromClientToServer, MessageFromServerToClient};
use std::io::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub async fn run_client(args: &Args) -> () {
    let (tx_to_ui, rx_to_ui) = async_channel::bounded::<MessageFromServerToClient>(
        MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
    );
    let (tx_from_ui, rx_from_ui) = async_channel::bounded::<MessageFromClientToServer>(
        MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
    );

    let shutdown_marker = Arc::new(AtomicBool::new(false));

    let network_task = tokio::spawn(run_network_task(
        args.clone(),
        tx_to_ui,
        rx_from_ui,
        Arc::clone(&shutdown_marker),
    ));
    let shutdown_marker_sdt = Arc::clone(&shutdown_marker);
    let shutdown_task = tokio::spawn(async move {
        // listen for ctrl-c
        tokio::signal::ctrl_c().await?;

        shutdown_marker_sdt.store(true, Ordering::SeqCst);

        Ok::<_, Error>(())
    });

    // async runtime stuff started, the display task doesn't like being inside tokio, so it comes after and takes shutdown orders via Arc
    tokio::spawn(async move {
        match tokio::try_join!(network_task, shutdown_task) {
            Err(_) => error!("Error in at least one client task"),
            Ok(_) => info!("All client tasks closed successfully"),
        };
    });

    run_display_task(
        args.clone(),
        rx_to_ui,
        tx_from_ui,
        Arc::clone(&shutdown_marker),
    );
}
