use rust_to_ts_types::TypescriptSerializable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, TypescriptSerializable)]
pub struct DisplayEntry {
    pub bib: u32,
    pub name: String,
    pub round: u16,
    pub max_rounds: u16,
}
