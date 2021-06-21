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

pub fn copy(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 2 {
        if let RespType::RBulkString(source) = &cmd[1] {
            if let RespType::RBulkString(destination) = &cmd[2] {
                if let Ok(write_guard) = database.write() {
                    let mut db = write_guard;
                    if cmd.len() == 4 {
                        if let RespType::RBulkString(replace) = &cmd[3] {
                            if replace == "replace" {
                                let res =
                                    db.copy(source.to_string(), destination.to_string(), true);
                                if let Some(()) = res {
                                    return RespType::RInteger(1);
                                } else {
                                    return RespType::RInteger(0);
                                }
                            }
                        }
                    } else {
                        let res = db.copy(source.to_string(), destination.to_string(), false);
                        if let Some(()) = res {
                            return RespType::RInteger(1);
                        } else {
                            return RespType::RInteger(0);
                        }
                    }
                }
            }
        }
    }
    RespType::RInteger(0)
}

pub fn exists(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut key_found = 0;
    if cmd.len() > 1 {
        for n in cmd.iter().skip(1) {
            if let RespType::RBulkString(actual_key) = n {
                if let Some(_key) = database.read().unwrap().search_item_by_key(actual_key) {
                    key_found = 1;
                }
            }
        }
    }
    RespType::RInteger(key_found)
}

pub fn persist(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if let RespType::RBulkString(key) = &cmd[1] {
        if database.write().unwrap().persist(key.to_string()) {
            return RespType::RInteger(1);
        } else {
            return RespType::RInteger(0);
        }
    }
    RespType::RInteger(0)
}

pub fn rename(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(actual_key) = &cmd[1] {
            let mut new_database = database.write().unwrap();
            if let RespType::RBulkString(new_key) = &cmd[2] {
                new_database.rename_key(actual_key.to_string(), new_key.to_string())
            }
        }
    }
    RespType::RBulkString("OK".to_string())
}

pub fn expire(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() != 2{
        RespType::RInteger(0);
    }else{
    if let RespType::RBulkString(key) = &cmd[1] {
        let mut db = database.write().unwrap();
        if let RespType::RBulkString(timeout) = &cmd[2] {
            let result = db.expire_key(key.to_string(), timeout);
            if result{
                return RespType::RInteger(1)
            }else{
                return RespType::RInteger(0)
            }
        }
    }}
    RespType::RInteger(0)
}

//     for n in cmd.iter().skip(1) {
//         //            key_in_db = database.search_by_key()
//         if let RespType::RBulkString(actual_key) = n {
//             if let Some(_key) = database.read().unwrap().search_item_by_key(actual_key) {
//                 key_found = 1;
//             }
//         }
//     }
// }
