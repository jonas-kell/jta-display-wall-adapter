mod bitmap;
mod parts;
mod rasterizing;
mod rendering;

pub use parts::client::run_client;

pub const TARGET_FPS: u64 = 60;
pub const REPORT_FRAME_LOGS_EVERY_SECONDS: u64 = 5;
pub const FRAME_TIME_NS: u64 = 1_000_000_000 / TARGET_FPS as u64;
