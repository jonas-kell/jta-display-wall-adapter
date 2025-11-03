use crate::args::Args;
use crate::nrbf::{decode_single_nrbf, image_response, pre_response};
use std::io;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::time;

/// Start server
pub async fn run_server(args: &Args) -> () {
    let address_display_program: SocketAddr = format!(
        "{}:{}",
        args.target_address_display_program, args.listen_port_display_program
    )
    .parse()
    .expect("Invalid display_program address");

    let own_addr_timing: SocketAddr = format!("0.0.0.0:{}", args.listen_port_display_program)
        .parse()
        .expect("Invalid listen address");

    info!("Talking to {} as display program", address_display_program);
    info!(
        "Listening self to the timing program on {}",
        own_addr_timing
    );

    // Define tcp listeners
    let tcp_listener_shutdown_marker = Arc::new(AtomicBool::new(false));

    let tcp_listener_server_instance = tcp_listener_server(
        Arc::clone(&tcp_listener_shutdown_marker),
        own_addr_timing,
        address_display_program,
    );

    // spawn the async runtimes in parallel
    let tcp_listener_server_task = tokio::spawn(tcp_listener_server_instance);
    let shutdown_task = tokio::spawn(async move {
        // listen for ctrl-c
        tokio::signal::ctrl_c().await.unwrap();

        tcp_listener_shutdown_marker.store(true, Ordering::SeqCst);
    });

    // Wait for all tasks to complete
    // https://github.com/actix/actix-web/issues/2739#issuecomment-1107638674
    match tokio::try_join!(tcp_listener_server_task, shutdown_task) {
        Err(_) => error!("Error in at least one listening task"),
        Ok(_) => info!("All listeners closed successfully"),
    };
}

pub async fn tcp_listener_server(
    shutdown_marker: Arc<AtomicBool>,
    listen_addr: SocketAddr,
    target_addr: SocketAddr,
) -> io::Result<()> {
    let listener = TcpListener::bind(listen_addr).await?;
    debug!("TCP listener started on {}", listen_addr);

    loop {
        if shutdown_marker.load(Ordering::SeqCst) {
            debug!("Shutdown requested, stopping listener on {}", listen_addr);
            break;
        }

        // Wait for new connection with timeout so we can check shutdown flag periodically
        match time::timeout(Duration::from_millis(10000), listener.accept()).await {
            Ok(Ok((inbound, client_addr))) => {
                debug!("Accepted connection from {}", client_addr);

                let target_addr = target_addr.clone();
                let shutdown_marker = shutdown_marker.clone();

                tokio::spawn(async move {
                    match TcpStream::connect(target_addr).await {
                        Ok(outbound) => {
                            debug!("Connected to target {}", target_addr);
                            if let Err(e) =
                                transfer_bidirectional(inbound, outbound, shutdown_marker.clone())
                                    .await
                            {
                                error!(
                                    "Error during transfer between {} and {}: {}",
                                    client_addr, target_addr, e
                                );
                            } else {
                                debug!(
                                    "Closed connection between {} and {}",
                                    client_addr, target_addr
                                );
                            }
                        }
                        Err(e) => error!("Failed to connect to target {}: {}", target_addr, e),
                    }
                });
            }
            Ok(Err(e)) => error!("Accept error: {}", e),
            Err(_) => {
                // expected on timeout, just loop
                trace!("No incoming TCP connection within timeout interval");
            }
        }
    }

    Ok(())
}

async fn transfer_bidirectional(
    mut inbound: TcpStream,
    mut outbound: TcpStream,
    shutdown_marker: Arc<AtomicBool>,
) -> io::Result<()> {
    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = async {
        let mut buf = [0u8; 65536];
        loop {
            if shutdown_marker.load(Ordering::SeqCst) {
                debug!("Shutdown marker set, breaking client→server transfer");
                break;
            }
            let n = match ri.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => return Err(e),
            };
            // match decode_single_nrbf(&buf[..n]) {
            //     Err(err) => trace!("Error when Decoding Inbound Communication: {}", err),
            //     Ok(parsed) => trace!("Decoded Inbound Communication: {:?}", parsed),
            // }

            wo.write_all(&buf[..n]).await?;
        }
        Ok::<_, io::Error>(())
    };

    let server_to_client = async {
        let mut buf = [0u8; 65536];
        loop {
            if shutdown_marker.load(Ordering::SeqCst) {
                debug!("Shutdown marker set, breaking server→client transfer");
                break;
            }
            let n = match ro.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => return Err(e),
            };
            match decode_single_nrbf(&buf[..n]) {
                Err(err) => trace!("Error when Decoding Return Communication: {}", err),
                Ok(parsed) => trace!("Decoded Return Communication: {:?}", parsed),
            }

            let pre_vec = pre_response();
            let vec = image_response();

            // wi.write_all(&buf[..n]).await?;

            wi.write_all(&pre_vec).await?;
            wi.write_all(&vec).await?;
        }
        Ok::<_, io::Error>(())
    };

    tokio::select! {
        r = client_to_server => r?,
        r = server_to_client => r?,
    }

    Ok(())
}
