use std::time::{Duration, Instant};

use crate::{
    times::DayTime,
    wind::format::{WindMeasurementType, WindMessageBroadcast},
};

pub struct WindStateManager {
    internal_date_time: Option<(Instant, DayTime)>,
    internal_action_tracker: Option<(Instant, WindMeasurementType)>,
}

impl WindStateManager {
    pub fn new() -> Self {
        Self {
            internal_date_time: None,
            internal_action_tracker: None,
        }
    }

    pub fn update_internal_time(&mut self, new_time: DayTime) {
        self.internal_date_time = Some((Instant::now(), new_time));
    }

    pub fn populate_broadcast_message(
        &mut self,
        message: WindMessageBroadcast,
    ) -> WindMessageBroadcast {
        match message {
            WindMessageBroadcast::Started(mut started_meas) => {
                // store that a measurement action started
                self.internal_action_tracker = Some((Instant::now(), started_meas.ms_type.clone()));

                // populate time if possible
                if let Some((inst, dt)) = &self.internal_date_time {
                    let supposed_current_time =
                        dt.add_duration(Instant::now().saturating_duration_since(inst.clone()));

                    started_meas.time = Some(supposed_current_time);
                }

                WindMessageBroadcast::Started(started_meas)
            }
            WindMessageBroadcast::Measured(mut meas) => {
                // populate time if possible
                if let Some((inst, dt)) = &self.internal_date_time {
                    let supposed_current_time =
                        dt.add_duration(Instant::now().saturating_duration_since(inst.clone()));

                    meas.time = Some(supposed_current_time);
                }

                // populate measurement type and update local state
                if let Some((inst, meas_type)) = &self.internal_action_tracker {
                    // longest measurement should take 13 seconds plus a bit leeway
                    if Instant::now().saturating_duration_since(inst.clone())
                        < Duration::from_secs(16)
                        && meas.probable_measurement_type
                            == WindMeasurementType::UnidentifiedMeasurement
                        && meas_type != &WindMeasurementType::Polling
                    {
                        meas.probable_measurement_type = meas_type.clone();
                    }

                    self.internal_action_tracker = None; // the type reference is consumed in any case, as soon as a measurement is received
                }

                WindMessageBroadcast::Measured(meas)
            }
        }
    }
}
