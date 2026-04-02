mod idcapture_format;
mod parts;

pub mod format {
    pub use super::idcapture_format::*;
}

pub use parts::run_idcapture_server;
