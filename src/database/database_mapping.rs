// to avoid mapping this into the database for real (as we only basically process the datatypes in rust and make really no computations on the database) we just store everything serealized

use crate::database::db::DatabaseError;
use crate::database::schema::heat_starts;
use crate::database::DatabaseManager;
use crate::server::xml_types::HeatStart;
use diesel::associations::HasTable;
use diesel::prelude::*;
use serde::Serialize;

pub trait DatabaseSerializable: Sized + Serialize {
    type DbTable: Table + HasTable<Table = Self::DbTable>;
    type DbModel: Insertable<Self::DbTable>;

    fn serialize_for_database(self) -> Result<Self::DbModel, DatabaseError>;

    fn store_to_database(self, manager: &DatabaseManager) -> Result<(), DatabaseError>;
}

macro_rules! impl_database_serializable {
    ($domain:ty, $db_model:ty, $table:ty) => {
        impl DatabaseSerializable for $domain {
            type DbModel = $db_model;
            type DbTable = $table;

            fn serialize_for_database(self) -> Result<Self::DbModel, DatabaseError> {
                Ok(Self::DbModel {
                    id: self.id.to_string(),
                    data: serde_json::to_string(&self)?,
                })
            }

            fn store_to_database(self, manager: &DatabaseManager) -> Result<(), DatabaseError> {
                let mut conn = manager.get_connection()?;
                let db_model = self.serialize_for_database()?;
                diesel::insert_into(<$table>::table())
                    .values(db_model)
                    .execute(&mut conn)?;

                Ok(())
            }
        }
    };
}

#[derive(Insertable, Queryable)]
#[diesel(table_name = heat_starts)]
pub struct HeatStartDatabase {
    id: String,
    data: String,
}
impl_database_serializable!(HeatStart, HeatStartDatabase, heat_starts::table);
