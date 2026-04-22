pub mod bib_detection;
mod camera_program_datatypes;
pub mod comm_channel;
mod parts;
mod xml_serial;

pub use parts::server::run_server;
pub mod camera_program_types {
    pub use super::camera_program_datatypes::*;
}
pub use parts::{
    audio_types,
    export_functions,
    CompetitorEvaluatedBibServer, // TODO package these expoerts more pretty
    MessageFromBibServer,
    MessageToBibServer,
    RaceHasStartedBibServer,
    SeekForTimeBibServer,
};
