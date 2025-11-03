use crate::args::Args;
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

    let address_timing_program: SocketAddr = format!(
        "{}:{}",
        args.target_address_timing_program, args.listen_port_timing_program
    )
    .parse()
    .expect("Invalid timing_program address");

    let own_addr_display: SocketAddr = format!("0.0.0.0:{}", args.listen_port_display_program)
        .parse()
        .expect("Invalid listen address");

    let own_addr_timing: SocketAddr = format!("0.0.0.0:{}", args.listen_port_timing_program)
        .parse()
        .expect("Invalid listen address");

    info!("Talking to {} as display program", address_display_program);
    info!("Talking to {} as timing program", address_timing_program);
    info!(
        "Listening self to the timing program on {} and to the display program on {}",
        own_addr_timing, own_addr_display
    );

    // Define tcp listeners
    let tcp_listener_shutdown_marker = Arc::new(AtomicBool::new(false));

    let tcp_listener_display_to_timing = tcp_listener_server(
        Arc::clone(&tcp_listener_shutdown_marker),
        own_addr_display,
        address_timing_program,
    );
    let tcp_listener_timing_to_display = tcp_listener_server(
        Arc::clone(&tcp_listener_shutdown_marker),
        own_addr_timing,
        address_display_program,
    );

    // spawn the async runtimes in parallel
    let tcp_listener_server_display_to_timing = tokio::spawn(tcp_listener_display_to_timing);
    let tcp_listener_server_timing_to_display = tokio::spawn(tcp_listener_timing_to_display);
    let shutdown_task = tokio::spawn(async move {
        // listen for ctrl-c
        tokio::signal::ctrl_c().await.unwrap();

        tcp_listener_shutdown_marker.store(true, Ordering::SeqCst);
    });

    // Wait for all tasks to complete
    // https://github.com/actix/actix-web/issues/2739#issuecomment-1107638674
    match tokio::try_join!(
        tcp_listener_server_display_to_timing,
        tcp_listener_server_timing_to_display,
        shutdown_task
    ) {
        Err(_) => error!("Error in at least one listening task"),
        Ok(_) => info!("All listeners closed successfully"),
    };
}

async fn tcp_listener_server(
    shutdown_marker: Arc<AtomicBool>,
    listen_addr: SocketAddr,
    target_addr: SocketAddr,
) -> std::io::Result<()> {
    let listener = match TcpListener::bind(listen_addr).await {
        Err(e) => {
            error!("Error creating TCP listener: {}", e);
            return Err(e);
        }
        Ok(s) => s,
    };

    loop {
        if shutdown_marker.load(Ordering::SeqCst) {
            debug!("Shutdown marker set. Stopping TCP listener.");
            break;
        }

        // Accept new connection (with timeout to allow periodic shutdown checks)
        match time::timeout(Duration::from_millis(100000), listener.accept()).await {
            Ok(Ok((mut inbound, src_addr))) => {
                debug!("Accepted connection from {}", src_addr);

                let target_addr = target_addr.clone();
                let shutdown_marker = shutdown_marker.clone();

                tokio::spawn(async move {
                    match TcpStream::connect(target_addr).await {
                        Ok(mut outbound) => {
                            debug!("Connected to target {}", target_addr);

                            let mut buffer = [0u8; 65536];
                            loop {
                                if shutdown_marker.load(Ordering::SeqCst) {
                                    debug!(
                                        "Shutdown marker set. Closing connection from {}",
                                        src_addr
                                    );
                                    break;
                                }

                                match inbound.read(&mut buffer).await {
                                    Ok(0) => {
                                        debug!("Connection closed by {}", src_addr);
                                        break;
                                    }
                                    Ok(n) => {
                                        if let Err(e) = outbound.write_all(&buffer[..n]).await {
                                            error!("Write error to {}: {}", target_addr, e);
                                            break;
                                        }
                                        trace!(
                                            "Forwarded {} bytes from {} to {}",
                                            n,
                                            src_addr,
                                            target_addr
                                        );
                                    }
                                    Err(e) => {
                                        error!("Read error from {}: {}", src_addr, e);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => error!("Failed to connect to target {}: {}", target_addr, e),
                    }
                });
            }
            Ok(Err(e)) => error!("Accept error on {}: {}", listen_addr, e),
            Err(_) => trace!("No TCP connection accepted in last interval."),
        }
    }

    Ok(())
}
