#[macro_use]
extern crate log;

use crate::args::{Args, Mode};
use clap::Parser;
use client::run_client;
use server::run_server;

mod args;
mod bitmap;
mod client;
mod forwarding;
mod hex;
mod instructions;
mod interface;
mod nrbf;
mod rasterizing;
mod rendering;
mod server;
mod xml_serial;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var(
        "RUST_LOG",
        "debug,actix=off,reqwest=off,hyper=off,mio=off,wgpu_core=info,wgpu_hal=info,naga=info,calloop=info",
    );

    let args = Args::parse();
    if args.verbose {
        std::env::set_var(
            "RUST_LOG",
            "trace,actix=off,reqwest=off,hyper=off,mio=off,wgpu_core=info,wgpu_hal=info,naga=info,calloop=info",
        );
        // more logs!!
    }

    env_logger::init();

    match args.mode {
        Mode::Server => run_server(&args).await,
        Mode::Client => run_client(&args).await,
    }

    Ok(())
}
