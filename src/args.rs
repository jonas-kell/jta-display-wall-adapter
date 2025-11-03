use clap::{Parser, ValueEnum};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Mode of operation: 'server' or 'client'
    #[arg(value_enum)]
    pub mode: Mode,
    /// Address of where the timing program lives (like "127.0.0.1")
    #[arg(long)]
    pub target_address_timing_program: String,
    /// Address of where the display program lives (like "127.0.0.1")
    #[arg(long)]
    pub target_address_display_program: String,
    /// Port where the application should listen to the timing program
    #[arg(long, default_value_t = String::from("52426"))]
    pub listen_port_timing_program: String,
    /// Port where the application should listen to the display program
    #[arg(long, default_value_t = String::from("18690"))]
    pub listen_port_display_program: String,
    /// Verbosity (-v for verbose mode)
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub verbose: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Mode {
    Server,
    Client,
}
