// to avoid mapping this into the database for real (as we only basically process the datatypes in rust and make really no computations on the database) we just store everything serealized

use crate::database::db::DatabaseError;
use crate::database::schema::heat_starts;
use crate::database::DatabaseManager;
use crate::server::xml_types::HeatStart;
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait DatabaseSerializable: Sized + Serialize + for<'a> Deserialize<'a> {
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
                    .values(db_model)
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

#[derive(Insertable, Queryable, Identifiable)]
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
