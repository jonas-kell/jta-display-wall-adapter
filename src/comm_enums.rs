use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientView {
    Advertisements,
    StartList,
    Timing,
    ResultList,
    Other,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ButtonAction {
    ToStartList,
    ToTiming,
    ToResultList,
    AdvanceRun,
    PreviousRun,
    Advertisements,
}
