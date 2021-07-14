use crate::domain::implementations::database::Database;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

const TIME_TO_SAVE_IN_FILE: u64 = 60 * 5; // in secs

pub fn dump_to_file(database: Arc<RwLock<Database>>) {
    loop {
        save_database(database.clone());
        thread::sleep(Duration::from_secs(TIME_TO_SAVE_IN_FILE));
    }
}

/// Guarda la base de datos en el archivo especificado en la configuracion.
///
/// Recibe una base de datos de tipo Database protegida por un RwLock
/// y guarda la información en su correspondiente archivo
fn save_database(database: Arc<RwLock<Database>>) {
    println!("Saving database to dump");
    let x = Arc::try_unwrap(database).unwrap_err();
    match x.try_read() {
        Ok(n) => n.save_items_to_file(),
        Err(_) => println!("Database couldn't be saved into file"),
    };
}
