use serde::{Deserialize, Serialize};
// use rust_to_ts_types::TypescriptSerializable;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum IDCaptureMessage {
    TMP(bool),
}
