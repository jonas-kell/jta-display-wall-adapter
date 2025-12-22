// to avoid mapping this into the database for real (as we only basically process the datatypes in rust and make really no computations on the database) we just store everything serealized
// TODO sqlite has a json type for efficiency

use crate::database::db::DatabaseError;
use crate::database::schema::{
    database_state, heat_evaluations, heat_false_starts, heat_finishes, heat_intermediates,
    heat_results, heat_start_lists, heat_starts, heat_wind_missings, heat_winds,
    internal_wind_measurements, internal_wind_readings, permanent_storage,
};
use crate::database::DatabaseManager;
use crate::server::camera_program_types::{
    CompetitorEvaluated, HeatData, HeatFalseStart, HeatFinish, HeatIntermediate, HeatResult,
    HeatStart, HeatStartList, HeatWind, HeatWindMissing,
};
use crate::times::DayTime;
use crate::wind::format::{StartedWindMeasurement, WindMeasurement};
use chrono::Utc;
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime};
use clap::crate_version;
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait DatabaseSerializable: Serialize + for<'a> Deserialize<'a> {
    type DbTable: Table + HasTable<Table = Self::DbTable>;
    type DbModel: Insertable<Self::DbTable>;

    fn serialize_for_database(&self) -> Result<Self::DbModel, DatabaseError>;

    fn store_to_database(self, manager: &DatabaseManager) -> Result<(), DatabaseError>;

    fn get_from_database_by_id(id: Uuid, manager: &DatabaseManager) -> Result<Self, DatabaseError>;

    fn get_all_from_database(manager: &DatabaseManager) -> Result<Vec<Self>, DatabaseError>;
}

macro_rules! impl_database_serializable {
    ($domain:ty, $db_model:ty, $table:ty, $id:expr, $ser_cb:expr) => {
        impl TryFrom<$db_model> for $domain {
            fn try_from(value: $db_model) -> Result<Self, Self::Error> {
                Ok(serde_json::from_str(&value.data)?)
            }

            type Error = DatabaseError;
        }

        impl DatabaseSerializable for $domain {
            type DbModel = $db_model;
            type DbTable = $table;

            fn serialize_for_database(&self) -> Result<Self::DbModel, DatabaseError> {
                $ser_cb(self) // callback here is more versatile than hardcoding what the id parameter is
            }

            fn store_to_database(self, manager: &DatabaseManager) -> Result<(), DatabaseError> {
                let mut conn = manager.get_connection()?;
                let db_model = self.serialize_for_database()?;
                diesel::insert_into(<$table>::table())
                    .values(&db_model)
                    .on_conflict($id)
                    .do_update()
                    .set(&db_model)
                    .execute(&mut conn)?;
                // permanent storage
                let name = String::from(
                    std::any::type_name::<$domain>()
                        .rsplitn(2, "::")
                        .next()
                        .unwrap_or("NOTHING"),
                );
                let perm = PermanentStorageDatabase {
                    id: Uuid::new_v4().to_string(),
                    name_key: name,
                    stored_at: Utc::now().naive_utc(),
                    data: db_model.data,
                };
                diesel::insert_into(permanent_storage::table::table())
                    .values(&perm)
                    .on_conflict_do_nothing()
                    .execute(&mut conn)?;

                Ok(())
            }

            fn get_from_database_by_id(
                id: Uuid,
                manager: &DatabaseManager,
            ) -> Result<Self, DatabaseError> {
                let mut conn = manager.get_connection()?;
                let data: Self::DbModel = <$table>::table()
                    .filter($id.eq(id.to_string()))
                    .first(&mut conn)?;

                Self::try_from(data)
            }

            fn get_all_from_database(
                manager: &DatabaseManager,
            ) -> Result<Vec<Self>, DatabaseError> {
                let mut conn = manager.get_connection()?;
                let data = <$table>::table().load::<Self::DbModel>(&mut conn)?;

                let collected = match data
                    .into_iter()
                    .map(|h| Self::try_from(h))
                    .collect::<Result<Vec<Self>, DatabaseError>>()
                {
                    Ok(a) => a,
                    Err(e) => return Err(e),
                };

                Ok(collected)
            }
        }
    };
}

#[derive(Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = heat_starts)]
pub struct HeatStartDatabase {
    id: String,
    data: String,
}
impl_database_serializable!(
    HeatStart,
    HeatStartDatabase,
    heat_starts::table,
    heat_starts::id,
    |self_obj: &HeatStart| Ok(HeatStartDatabase {
        id: self_obj.id.to_string(),
        data: serde_json::to_string(self_obj)?,
    })
);

#[derive(Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = heat_start_lists)]
pub struct HeatStartListDatabase {
    id: String,
    data: String,
}
impl_database_serializable!(
    HeatStartList,
    HeatStartListDatabase,
    heat_start_lists::table,
    heat_start_lists::id,
    |self_obj: &HeatStartList| Ok(HeatStartListDatabase {
        id: self_obj.id.to_string(),
        data: serde_json::to_string(self_obj)?,
    })
);

#[derive(Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = heat_false_starts)]
pub struct HeatFalseStartDatabase {
    id: String,
    data: String,
}
impl_database_serializable!(
    HeatFalseStart,
    HeatFalseStartDatabase,
    heat_false_starts::table,
    heat_false_starts::id,
    |self_obj: &HeatFalseStart| Ok(HeatFalseStartDatabase {
        id: self_obj.id.to_string(),
        data: serde_json::to_string(self_obj)?,
    })
);

#[derive(Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = heat_intermediates)]
pub struct HeatIntermediateDatabase {
    id: String,
    belongs_to_id: String,
    data: String,
}
impl_database_serializable!(
    HeatIntermediate,
    HeatIntermediateDatabase,
    heat_intermediates::table,
    heat_intermediates::id,
    |self_obj: &HeatIntermediate| Ok(HeatIntermediateDatabase {
        id: Uuid::new_v4().to_string(), // multiple per run are possible
        belongs_to_id: self_obj.id.to_string(),
        data: serde_json::to_string(self_obj)?,
    })
);

#[derive(Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = heat_winds)]
pub struct HeatWindDatabase {
    id: String,
    data: String,
}
impl_database_serializable!(
    HeatWind,
    HeatWindDatabase,
    heat_winds::table,
    heat_winds::id,
    |self_obj: &HeatWind| Ok(HeatWindDatabase {
        id: self_obj.id.to_string(),
        data: serde_json::to_string(self_obj)?,
    })
);

#[derive(Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = heat_wind_missings)]
pub struct HeatWindMissingDatabase {
    id: String,
    data: String,
}
impl_database_serializable!(
    HeatWindMissing,
    HeatWindMissingDatabase,
    heat_wind_missings::table,
    heat_wind_missings::id,
    |self_obj: &HeatWindMissing| Ok(HeatWindMissingDatabase {
        id: self_obj.id.to_string(),
        data: serde_json::to_string(self_obj)?,
    })
);

#[derive(Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = heat_finishes)]
pub struct HeatFinishDatabase {
    id: String,
    data: String,
}
impl_database_serializable!(
    HeatFinish,
    HeatFinishDatabase,
    heat_finishes::table,
    heat_finishes::id,
    |self_obj: &HeatFinish| Ok(HeatFinishDatabase {
        id: self_obj.id.to_string(),
        data: serde_json::to_string(self_obj)?,
    })
);

#[derive(Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = heat_evaluations)]
pub struct HeatEvaluationDatabase {
    id: String,
    belongs_to_id: String,
    data: String,
}
impl_database_serializable!(
    CompetitorEvaluated,
    HeatEvaluationDatabase,
    heat_evaluations::table,
    heat_evaluations::id,
    |self_obj: &CompetitorEvaluated| Ok(HeatEvaluationDatabase {
        id: Uuid::new_v4().to_string(), // multiple per run are possible
        belongs_to_id: self_obj.id.to_string(),
        data: serde_json::to_string(self_obj)?,
    })
);

#[derive(Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = heat_results)]
pub struct HeatResultDatabase {
    id: String,
    data: String,
}
impl_database_serializable!(
    HeatResult,
    HeatResultDatabase,
    heat_results::table,
    heat_results::id,
    |self_obj: &HeatResult| Ok(HeatResultDatabase {
        id: self_obj.id.to_string(),
        data: serde_json::to_string(self_obj)?,
    })
);

#[derive(Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = internal_wind_readings)]
pub struct InternalWindReadingsDatabase {
    id: String,
    data: String,
    wind_meas_time: Option<NaiveDateTime>,
    stored_at_local: NaiveDateTime,
}
impl_database_serializable!(
    WindMeasurement,
    InternalWindReadingsDatabase,
    internal_wind_readings::table,
    internal_wind_readings::id,
    |self_obj: &WindMeasurement| Ok(InternalWindReadingsDatabase {
        id: Uuid::new_v4().to_string(),
        data: serde_json::to_string(self_obj)?,
        wind_meas_time: transform_time(&self_obj.time),
        stored_at_local: Local::now().naive_local(),
    })
);

#[derive(Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = internal_wind_measurements)]
pub struct InternalWindMeasurementsDatabase {
    id: String,
    data: String,
    wind_meas_time: Option<NaiveDateTime>,
    stored_at_local: NaiveDateTime,
}
impl_database_serializable!(
    StartedWindMeasurement,
    InternalWindMeasurementsDatabase,
    internal_wind_measurements::table,
    internal_wind_measurements::id,
    |self_obj: &StartedWindMeasurement| Ok(InternalWindMeasurementsDatabase {
        id: Uuid::new_v4().to_string(),
        data: serde_json::to_string(self_obj)?,
        wind_meas_time: transform_time(&self_obj.time),
        stored_at_local: Local::now().naive_local(),
    })
);

fn transform_time(time: &Option<DayTime>) -> Option<NaiveDateTime> {
    match time {
        Some(time) => {
            let hours: u16 = time.hours;
            let minutes: u16 = time.minutes;
            let seconds: u16 = time.seconds;
            let fractional_part_in_ten_thousands: Option<u32> =
                time.fractional_part_in_ten_thousands;

            let nanos = fractional_part_in_ten_thousands
                .map(|f| (f as u64) * 100_000) // 1/10_000 second = 100,000 nanoseconds
                .unwrap_or(0) as u32;

            // Build NaiveTime
            let naive_time = match NaiveTime::from_hms_nano_opt(
                hours as u32,
                minutes as u32,
                seconds as u32,
                nanos,
            ) {
                Some(a) => a,
                None => return None,
            };

            let today: NaiveDate = Local::now().date_naive();

            return Some(NaiveDateTime::new(today, naive_time));
        }
        None => None,
    }
}

#[derive(Insertable, Queryable, Identifiable)]
#[diesel(table_name = permanent_storage)]
struct PermanentStorageDatabase {
    id: String,
    name_key: String,
    stored_at: NaiveDateTime,
    data: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PermanentlyStoredDataset {
    name_key: String,
    stored_at: NaiveDateTime,
    data: String,
}

pub fn get_log_limited(
    limit: Option<u32>,
    manager: &DatabaseManager,
) -> Result<Vec<PermanentlyStoredDataset>, DatabaseError> {
    let mut conn = manager.get_connection()?;
    let data = permanent_storage::table::table()
        .order(permanent_storage::stored_at.desc())
        .limit(limit.map(|a| a as i64).unwrap_or(i64::MAX))
        .load::<PermanentStorageDatabase>(&mut conn)?;

    Ok(data
        .into_iter()
        .map(|h| PermanentlyStoredDataset {
            data: h.data,
            name_key: h.name_key,
            stored_at: h.stored_at,
        })
        .collect::<Vec<PermanentlyStoredDataset>>())
}

pub fn get_heat_data(id: Uuid, manager: &DatabaseManager) -> Result<HeatData, DatabaseError> {
    let mut conn = manager.get_connection()?;
    let start_list = HeatStartList::get_from_database_by_id(id, manager)?;
    let id_str = id.to_string();

    let data_intermediates = heat_intermediates::table::table()
        .filter(heat_intermediates::belongs_to_id.eq(id_str.clone()))
        .load::<HeatIntermediateDatabase>(&mut conn)?;
    let intermediates_collected = data_intermediates
        .into_iter()
        .filter_map(|h| HeatIntermediate::try_from(h).ok())
        .collect::<Vec<HeatIntermediate>>();
    let data_evaluations = heat_evaluations::table::table()
        .filter(heat_evaluations::belongs_to_id.eq(id_str))
        .load::<HeatEvaluationDatabase>(&mut conn)?;
    let evaluations_collected = data_evaluations
        .into_iter()
        .filter_map(|h| CompetitorEvaluated::try_from(h).ok())
        .collect::<Vec<CompetitorEvaluated>>();

    // database error here basically always is not found error -> this is fine
    let heat_start = HeatStart::get_from_database_by_id(id, manager).ok();
    let heat_finish = HeatFinish::get_from_database_by_id(id, manager).ok();
    let heat_result = HeatResult::get_from_database_by_id(id, manager).ok();
    let heat_wind = HeatWind::get_from_database_by_id(id, manager).ok();

    return Ok(HeatData {
        meta: start_list.clone().into(),
        start_list: start_list,
        start: heat_start,
        intermediates: if intermediates_collected.is_empty() {
            None
        } else {
            Some(intermediates_collected)
        },
        evaluations: if evaluations_collected.is_empty() {
            None
        } else {
            Some(evaluations_collected)
        },
        finish: heat_finish,
        result: heat_result,
        wind: heat_wind,
    });
}

/// clear starts, intermediates, finish, results, winds, wind_missings, evaluations
pub fn purge_heat_data(id: Uuid, manager: &DatabaseManager) -> Result<(), DatabaseError> {
    let mut conn = manager.get_connection()?;

    // start list will egt ovrwritten immediately. Do not delete to avoid race conditions
    diesel::delete(heat_starts::table::table().filter(heat_starts::id.eq(id.to_string())))
        .execute(&mut conn)?;
    diesel::delete(
        heat_intermediates::table::table()
            .filter(heat_intermediates::belongs_to_id.eq(id.to_string())),
    )
    .execute(&mut conn)?;
    diesel::delete(heat_finishes::table::table().filter(heat_finishes::id.eq(id.to_string())))
        .execute(&mut conn)?;
    diesel::delete(heat_results::table::table().filter(heat_results::id.eq(id.to_string())))
        .execute(&mut conn)?;
    diesel::delete(heat_winds::table::table().filter(heat_winds::id.eq(id.to_string())))
        .execute(&mut conn)?;
    diesel::delete(
        heat_wind_missings::table::table().filter(heat_wind_missings::id.eq(id.to_string())),
    )
    .execute(&mut conn)?;
    diesel::delete(
        heat_evaluations::table::table().filter(heat_evaluations::belongs_to_id.eq(id.to_string())),
    )
    .execute(&mut conn)?;

    Ok(())
}

pub fn get_wind_readings(
    from: NaiveDateTime,
    to: NaiveDateTime,
    manager: &DatabaseManager,
) -> Result<Vec<WindMeasurement>, DatabaseError> {
    let mut conn = manager.get_connection()?;

    let data_wind = internal_wind_readings::table::table()
        .filter(internal_wind_readings::wind_meas_time.is_not_null())
        .filter(internal_wind_readings::wind_meas_time.ge(Some(from)))
        .filter(internal_wind_readings::wind_meas_time.le(Some(to)))
        .order(internal_wind_readings::wind_meas_time.asc())
        .load::<InternalWindReadingsDatabase>(&mut conn)?
        .into_iter()
        .filter_map(|iwr| WindMeasurement::try_from(iwr).ok())
        .collect();

    return Ok(data_wind);
}

#[derive(Insertable, Queryable, Identifiable)]
#[diesel(table_name = database_state)]
struct DatabaseStaticStateDatabase {
    id: i32,
    created_with_version: String,
    data: String,
}
impl TryFrom<DatabaseStaticState> for DatabaseStaticStateDatabase {
    type Error = String;

    fn try_from(value: DatabaseStaticState) -> Result<Self, Self::Error> {
        Ok(Self {
            id: 1,
            created_with_version: String::from(crate_version!()),
            data: serde_json::to_string(&value)
                .map_err(|e| format!("Could not serialize static data: {}", e.to_string()))?,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ApplicationMode {
    TrackCompetition,
    StreetLongRun,
    SprinterKing,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabaseStaticState {
    pub mode: ApplicationMode,
    pub date: NaiveDate,
    pub meet_id: Uuid,
}
impl TryFrom<DatabaseStaticStateDatabase> for DatabaseStaticState {
    type Error = String;

    fn try_from(value: DatabaseStaticStateDatabase) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value.data)
            .map_err(|e| format!("Could not deserialize static data: {}", e.to_string()))?)
    }
}

pub fn get_database_static_state(
    manager: &DatabaseManager,
) -> Result<DatabaseStaticState, DatabaseError> {
    let mut conn = manager.get_connection()?;

    let static_data = database_state::table::table()
        .filter(database_state::id.is(1))
        .first::<DatabaseStaticStateDatabase>(&mut conn)?;

    let current_version = String::from(crate_version!());

    if static_data.created_with_version != current_version {
        return Err(DatabaseError::new(format!("Could not read static data, as the database was created for a different program version. DB: {}, Current: {}", static_data.created_with_version, current_version)));
    }

    return Ok(static_data
        .try_into()
        .map_err(|e: String| Into::<DatabaseError>::into(e))?);
}

pub fn init_database_static_state(
    value: DatabaseStaticState,
    manager: &DatabaseManager,
) -> Result<DatabaseStaticState, DatabaseError> {
    match get_database_static_state(manager) {
        Ok(_) => {
            return Err(
                String::from("To initialize, the database static state must be empty!").into(),
            )
        }
        Err(_) => (),
    };

    let mut conn = manager.get_connection()?;

    let data: DatabaseStaticStateDatabase = match value.try_into() {
        Ok(d) => d,
        Err(e) => return Err(e.into()),
    };

    let _ = diesel::insert_into(database_state::table::table())
        .values(data)
        .execute(&mut conn)?;

    match get_database_static_state(manager) {
        Ok(a) => return Ok(a),
        Err(_) => Err(String::from("Even after initialization, database was found empty").into()),
    }
}
