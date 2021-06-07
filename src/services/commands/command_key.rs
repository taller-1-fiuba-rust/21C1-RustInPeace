// use crate::domain::entities::config::Config;
// use crate::domain::entities::message::WorkerMessage;
use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
use std::sync::{Arc, RwLock};
// use std::io::{Error, ErrorKind};
// use std::{net::SocketAddr, sync::mpsc::Sender};

pub fn del(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut n_key_deleted = 0;
    if cmd.len() == 1 {
        RespType::RInteger(n_key_deleted)
    } else {
        for n in cmd.iter().skip(1) {
            if let RespType::RBulkString(actual_key) = n {
                let mut new_database = database.write().unwrap();
                new_database.delete_key(actual_key.to_string());
                n_key_deleted += 1;
            }
        }
        RespType::RInteger(n_key_deleted)
    }
}

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
