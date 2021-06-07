use crate::domain::implementations::database::Database;
use std::sync::{Arc, RwLock};

pub fn copy(
    database: &Arc<RwLock<Database>>,
    source: String,
    destination: String,
    replace: bool,
) -> Option<()> {
    if let Ok(write_guard) = database.write() {
        let mut db = write_guard;
        return db.copy(source, destination, replace);
    }
    None
}