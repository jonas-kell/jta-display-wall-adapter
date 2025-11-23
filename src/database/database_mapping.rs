// to avoid mapping this into the database for real (as we only basically process the datatypes in rust and make really no computations on the database) we just store everything serealized
// TODO sqlite has a json type for efficiency

use crate::database::db::DatabaseError;
use crate::database::schema::{
    heat_false_starts, heat_start_lists, heat_starts, permanent_storage,
};
use crate::database::DatabaseManager;
use crate::server::xml_types::{HeatFalseStart, HeatStart, HeatStartList};
use chrono::NaiveDateTime;
use chrono::Utc;
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

#[derive(Insertable, Queryable, Identifiable)]
#[diesel(table_name = permanent_storage)]
struct PermanentStorageDatabase {
    id: String,
    name_key: String,
    stored_at: NaiveDateTime,
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
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
