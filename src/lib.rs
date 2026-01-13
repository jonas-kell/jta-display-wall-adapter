#[macro_use]
extern crate log;

mod args;
mod client;
mod database;
mod file;
mod helpers;
mod hex;
mod instructions;
mod interface;
mod json;
mod server;
mod times;
mod webserver;
mod wind;

pub fn open_webcontrol(args: &Args) {
    let _ = open::that(&format!(
        "http://localhost:{}/",
        args.internal_webcontrol_port
    ));
}

pub use args::{Args, Mode};
pub use client::run_client;
pub use server::run_server;
pub use wind::run_wind_server;

// export data for code generation
pub use webserver::{MessageFromWebControl, MessageToWebControl};
