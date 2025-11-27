use clap::{Parser, ValueEnum};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Mode of operation: 'server' or 'client'
    #[arg(value_enum)]
    pub mode: Mode,
    /// Verbosity (-v for verbose mode, -vv for extra verbose mode, -vvv for ALL logs)
    #[arg(short, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Port where the application should listen to the timing program
    #[arg(long, default_value_t = String::from("18690"))]
    pub listen_port: String,
    /// Listen to the timing program on listen_port
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub listen_to_timing_program: bool,
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
    /// Port where the internal communication between server and client takes place
    #[arg(long, default_value_t = String::from("5678"))]
    pub internal_communication_port: String,
    /// Port where the internal communication between server and webclient takes place
    #[arg(long, default_value_t = String::from("6789"))]
    pub internal_webcontrol_port: String,
    /// Address of where display client lives (for the server to talk to) (like "127.0.0.1")
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
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
    /// Number of time that passes, until the client sends the next frame
    #[arg(long, default_value_t = 500)]
    pub client_emits_frame_every_nr_of_ms: u64,
    /// Duration of one advertisement slideshow slide (initial for client, will get sent from server to client)
    #[arg(long, default_value_t = 2000)]
    pub slideshow_duration_nr_ms: u32,
    /// Duration of one advertisement slideshow slide transition (initial for client, will get sent from server to client)
    #[arg(long, default_value_t = 200)]
    pub slideshow_transition_duration_nr_ms: u32,
    /// Name of the database file (do not include /s that probably breaks shit)
    #[arg(long, default_value_t = String::from("db.db"))]
    pub database_file_name: String,
    /// Timing control variable for fireworks (initial for client, will get sent from server to client, can be set over webcontrol)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub fireworks_on_intermediate: bool,
    /// Timing control variable for fireworks (initial for client, will get sent from server to client, can be set over webcontrol)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub fireworks_on_finish: bool,
    /// Timing control variable for display of after comma decimals (initial for client, will get sent from server to client, can be set over webcontrol)
    #[arg(long, default_value_t = 2)]
    pub max_decimal_place_after_comma: i8,
    /// Timing control variable holding times (initial for client, will get sent from server to client, can be set over webcontrol)
    #[arg(long, default_value_t = 2000)]
    pub hold_time_ms: u32,
    /// Timing control variable for sound playback (initial for client, will get sent from server to client, can be set over webcontrol)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub play_sound_on_start: bool,
    /// Timing control variable for sound playback (initial for client, will get sent from server to client, can be set over webcontrol)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub play_sound_on_intermediate: bool,
    /// Timing control variable for sound playback (initial for client, will get sent from server to client, can be set over webcontrol)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub play_sound_on_finish: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Mode {
    Server,
    Client,
}

pub const MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS: usize = 100;
