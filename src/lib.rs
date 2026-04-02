#[macro_use]
extern crate log;

mod args;
mod client;
mod database;
mod file;
mod helpers;
mod hex;
mod idcapture;
mod instructions;
mod interface;
mod json;
mod productkey;
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
pub use idcapture::run_idcapture_server;
pub use productkey::initialize_product_key_system;
pub use server::run_server;
pub use wind::run_wind_server;

// export data for code generation
pub use webserver::{MessageFromWebControl, MessageToWebControl};
