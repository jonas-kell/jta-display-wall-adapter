use diesel::r2d2::ConnectionManager;
use diesel::result::Error as DieselError;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use r2d2;
use serde::Deserialize;
use std::fmt;
use std::num::TryFromIntError;
use std::path::Path;
use std::time::Duration;

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./src/database/migrations");

#[derive(Clone)]
pub struct DatabaseManager {
    pool: Pool,
}

impl DatabaseManager {
    pub fn init(db_file_path: &Path) -> Result<DatabaseManager, DatabaseError> {
        let database_url = format!("{}", db_file_path.display());
        let pool: Pool = {
            let manager = ConnectionManager::<SqliteConnection>::new(database_url);
            Pool::builder()
                .connection_timeout(Duration::from_secs(3))
                .max_size(4)
                .build(manager)
                .map_err(|e| DatabaseError::new(format!("Failed building database pool: {}", e)))?
        };

        let manager = DatabaseManager { pool };

        let mut conn = manager.get_connection()?;
        conn.run_pending_migrations(MIGRATIONS).map_err(|e| {
            DatabaseError::new(format!(
                "Failed to initialize database with migrations: {}",
                e
            ))
        })?;

        return Ok(manager);
    }

    pub fn get_connection(&self) -> Result<DbConnection, DatabaseError> {
        self.pool
            .get()
            .map_err(|e| DatabaseError::new(format!("Failed getting db connection: {}", e)))
    }
}

#[derive(Debug, Deserialize)]
pub struct DatabaseError {
    pub error_message: String,
}

impl DatabaseError {
    pub fn new(error_message: String) -> DatabaseError {
        DatabaseError { error_message }
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.error_message.as_str())
    }
}

impl From<DieselError> for DatabaseError {
    fn from(error: DieselError) -> DatabaseError {
        match error {
            DieselError::DatabaseError(_, err) => DatabaseError::new(err.message().to_string()),
            DieselError::NotFound => DatabaseError::new("The record was not found".to_string()),
            err => DatabaseError::new(format!("Unknown Diesel error: {}", err)),
        }
    }
}

impl From<TryFromIntError> for DatabaseError {
    fn from(error: TryFromIntError) -> DatabaseError {
        DatabaseError::new(format!("Integer conversion error: {}", error))
    }
}

impl From<DatabaseError> for std::io::Error {
    fn from(value: DatabaseError) -> Self {
        return std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Database error: {}", value.error_message),
        );
    }
}
