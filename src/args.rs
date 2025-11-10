use clap::{Parser, ValueEnum};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Mode of operation: 'server' or 'client'
    #[arg(value_enum)]
    pub mode: Mode,
    /// Port where the application should listen to the timing program
    #[arg(long, default_value_t = String::from("18690"))]
    pub listen_port: String,
    /// Verbosity (-v for verbose mode)
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub verbose: bool,
    /// Passthrough data to an external display program
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub passthrough_to_display_program: bool,
    /// Passthrough Port if external display program is used
    #[arg(long, default_value_t = String::from("18690"))]
    pub passthrough_port_display_program: String,
    /// Address of where the display program lives (like "127.0.0.1")
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub passthrough_address_display_program: String,
    /// Address of where camera program live (like "127.0.0.1")
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub address_camera_program: String,
    /// Port where the camera program has their timing endpoint
    #[arg(long, default_value_t = String::from("4445"))]
    pub camera_exchange_timing_port: String,
    /// Port where the camera program has their data endpoint
    #[arg(long, default_value_t = String::from("4446"))]
    pub camera_exchange_data_port: String,
    /// Port where the camera program has their xml endpoint
    #[arg(long, default_value_t = String::from("4447"))]
    pub camera_exchange_xml_port: String,
    /// Port where the internal communication between server and diplay client takes place
    #[arg(long, default_value_t = String::from("5678"))]
    pub internal_communication_port: String,
    /// Address of where display client lives (for the server to talk to) (like "127.0.0.1")
    #[arg(long, default_value_t = String::from("128.0.0.1"))]
    pub address_internal_communication: String,
    /// If the data that is incoming through should get hexdump-displayed
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub hexdump_incoming_communication: bool,
    /// If the data that gets passed through should get hexdump-displayed
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub hexdump_passthrough_communication: bool,
    /// Wait for connection until checking for shutdown in ms
    #[arg(long, default_value_t = 1000)]
    pub wait_ms_before_testing_for_shutdown: u64,
    /// Position of client window (initial for client, will get sent from server to client)
    #[arg(long, default_value_t = 200)]
    pub dp_pos_x: u32,
    /// Position of client window (initial for client, will get sent from server to client)
    #[arg(long, default_value_t = 200)]
    pub dp_pos_y: u32,
    /// Width of client window (initial for client, will get sent from server to client)
    #[arg(long, default_value_t = 720)]
    pub dp_width: u32,
    /// Height of client window (initial for client, will get sent from server to client)
    #[arg(long, default_value_t = 240)]
    pub dp_height: u32,
    /// The client will place a file in the running folder with coordinates where the window should be moved to (wayland sway script uses this to move window)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub emit_file_on_location_update: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Mode {
    Server,
    Client,
}
