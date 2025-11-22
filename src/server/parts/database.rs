use crate::{args::Args, database::DatabaseManager, file::make_sure_folder_exists};
use std::{io, path::Path};

pub fn database_test_task(args: Args) -> io::Result<()> {
    let path_folder = Path::new("database_container/");
    let path_file =
        Path::new("database_container/replaceme.db").with_file_name(&args.database_file_name);

    match make_sure_folder_exists(path_folder) {
        Ok(_) => {
            debug!("Database folder created or exists");
        }
        Err(e) => {
            error!("{}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Database folder could not be created: {}", e),
            ));
        }
    }

    let manager = DatabaseManager::init(&path_file)?;

    let _ = manager.get_connection();

    Ok(())
}
