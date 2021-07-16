//! Servicio para manejar la bajada a un archivo de la base de datos en memoria !

use crate::domain::implementations::database::Database;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

const TIME_TO_SAVE_IN_FILE: u64 = 60 * 2; // in secs

/// Itera infinitamente y cada 2 minutos hace una bajada de los datos en memoria
/// a un archivo definido en el archivo de configuración.
///
pub fn dump_to_file(database: Arc<RwLock<Database>>) {
    loop {
        save_database(database.clone());
        thread::sleep(Duration::from_secs(TIME_TO_SAVE_IN_FILE));
    }
}

/// Guarda la base de datos en el archivo especificado en la configuracion.
///
/// Recibe una base de datos de tipo Database protegida por un RwLock
/// y guarda la información en su correspondiente archivo.
fn save_database(database: Arc<RwLock<Database>>) {
    println!("Saving database to dump");
    if let Ok(db) = Arc::try_unwrap(database).unwrap_err().try_read() {
        db.save_items_to_file()
    } else {
        println!("Database couldn't be saved into file");
    }
}
