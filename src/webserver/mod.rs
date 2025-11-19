mod interface;
mod routes;
mod static_files;
mod web;
use local_ip_address::local_ip;

pub fn get_local_ip() -> String {
    match local_ip() {
        Err(err) => String::from(format!("{}", err)),
        Ok(add) => add.to_string(),
    }
}

pub use interface::{MessageFromWebControl, MessageToWebControl};
pub use web::{webserver, HttpServerStateManager, Server};
