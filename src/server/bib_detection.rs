use crate::{
    database::{get_bib_data, get_heat_data, DatabaseManager},
    server::camera_program_types::HeatData,
    times::RaceTime,
};
use rust_to_ts_types::TypescriptSerializable;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, TypescriptSerializable)]
pub struct DisplayEntry {
    pub bib: u32,
    pub name: String,
    pub round: u16,
    pub max_rounds: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageFromBibServer {
    pub bib: u32,
    pub timestamp: f32, // seconds since midnight
                        // ... There are more fields, we do not parse
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompetitorEvaluatedBibServer {
    pub timestamp: f32, // seconds since midnight
    pub bib: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SeekForTimeBibServer {
    pub timestamp: f32, // seconds since midnight
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RaceHasStartedBibServer {
    pub id: String,
    pub timestamp: f32, // seconds since midnight
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "data")]
pub enum MessageToBibServer {
    CompetitorEvaluated(CompetitorEvaluatedBibServer),
    SeekForTime(SeekForTimeBibServer),
    RaceHasStarted(RaceHasStartedBibServer),
}

#[derive(Serialize, Deserialize, Clone, Debug, TypescriptSerializable)]
pub struct BibDataPoint {
    pub heat_id: Uuid,
    pub bib: u32,
    pub race_time: RaceTime,
    pub manual: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, TypescriptSerializable)]
pub struct BibEquivalence {
    pub heat_id: Uuid,
    pub finish_bib: u32,
    pub alternative_bib: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, TypescriptSerializable)]
pub struct BibEntryModeData {
    pub heat_data: HeatData,
    pub bib_data_points: Vec<BibDataPoint>,
    pub equivalences: Vec<BibEquivalence>,
}

pub fn generate_bib_data(
    bib_heat_id: Option<Uuid>,
    database_manager: &DatabaseManager,
) -> Option<BibEntryModeData> {
    let bib_heat_id = match bib_heat_id {
        None => return None,
        Some(a) => a,
    };

    let data = match get_heat_data(bib_heat_id.clone(), database_manager) {
        Ok(data) => data,
        Err(e) => {
            error!(
                "Failed to update bib state, because of: {}, in loading heat data",
                e.to_string()
            );
            return None;
        }
    };

    match get_bib_data(bib_heat_id, database_manager) {
        Ok((data_points, equivalences)) => {
            return Some(BibEntryModeData {
                heat_data: data,
                bib_data_points: data_points,
                equivalences,
            })
        }
        Err(e) => {
            error!("Failed to update bib state, because of: {}", e.to_string());
            return None;
        }
    }
}
