#[macro_use]
extern crate log;

use std::net::TcpListener;

use crate::args::{Args, Mode};
use clap::Parser;
use client::run_client;
use server::run_server;

mod args;
mod client;
mod file;
mod hex;
mod instructions;
mod interface;
mod server;
mod times;
mod webserver;

fn is_port_in_use(port: &str) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    TcpListener::bind(addr).is_err()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var(
        "RUST_LOG",
        "debug,actix=off,reqwest=off,hyper=off,mio=off,wgpu_core=info,wgpu_hal=info,naga=info,calloop=info,neli=info,tracing=off",
    );

    let args = Args::parse();
    if args.verbose {
        std::env::set_var(
            "RUST_LOG",
            "trace,actix=off,reqwest=off,hyper=off,mio=off,wgpu_core=info,wgpu_hal=info,naga=info,calloop=info,neli=info,tracing=off",
        );
        // more logs!!
    }

    env_logger::init();

    if matches!(args.mode, Mode::Server) {
        if is_port_in_use(&args.listen_port) || is_port_in_use(&args.internal_webcontrol_port) {
            error!("The program could not be started, as the tcp ports are already in use.");
            error!("Either an incompatible program is already running, or a second instance of this program is");

            info!("Opening control panel instead");
            // open the control panel as a helpful feature
            let _ = open::that(&format!(
                "http://localhost:{}/",
                args.internal_webcontrol_port
            ));
        }
    }

    match args.mode {
        Mode::Server => run_server(&args).await,
        Mode::Client => run_client(&args).await,
    }

    Ok(())
}
