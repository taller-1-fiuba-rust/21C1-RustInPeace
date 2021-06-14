use std::{convert::TryInto, sync::{Arc, RwLock}};
use crate::{domain::implementations::database::Database, services::utils::resp_type::RespType};

pub fn append(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 2 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            if let RespType::RBulkString(str_to_append) = &cmd[2] {
                return RespType::RInteger(db.append_string(key, str_to_append));
            }
        }
    }
    RespType::RInteger(0)
}

pub fn decrby(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 2 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            if let RespType::RBulkString(decr) = &cmd[2] {
                let number = decr.parse::<i64>();
                match number {
                    Ok(decr) => {
                        match db.decrement_key_by(key, decr) {
                            Ok(res) => {
                                return RespType::RInteger(res.try_into().unwrap());
                            }
                            Err(e) => {
                                return RespType::RError(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        return RespType::RError(e.to_string());
                    }
                }
            }
        }
    }
    RespType::RError(String::from("Invalid command decrby"))
}

pub fn get(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let db = database.read().unwrap();
            match db.get_value_by_key(key) {
                Some(str) => {
                    return RespType::RBulkString(str);
                }
                None => {
                    //nil - testear
                    return RespType::RNullBulkString();
                }
            }
        }
    }
    RespType::RError(String::from("Invalid command get"))
}

pub fn getdel(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            match db.getdel_value_by_key(key) {
                Some(str) => {
                    return RespType::RBulkString(str);
                }
                None => {
                    //nil - testear
                    return RespType::RNullBulkString();
                }
            }
        }
    }
    RespType::RError(String::from("Invalid command get"))
}

pub fn getset(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            if let RespType::RBulkString(new_value) = &cmd[2] {
                match db.getset_value_by_key(key, new_value) {
                    Some(str) => {
                        return RespType::RBulkString(str);
                    }
                    None => {
                        //nil - testear
                        return RespType::RNullBulkString();
                    }
                }
            }
        }
    }
    RespType::RError(String::from("Invalid command get"))
}