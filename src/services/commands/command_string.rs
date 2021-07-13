use crate::domain::entities::key_value_item::KeyAccessTime;
use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};
use crate::{domain::implementations::database::Database, services::utils::resp_type::RespType};
use std::vec;
use std::{
    convert::TryInto,
    sync::{Arc, RwLock},
};

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
                return match number {
                    Ok(decr) => match db.decrement_key_by(key, decr) {
                        Ok(res) => {
                            //falla si el nro es negativo
                            RespType::RInteger(res.try_into().unwrap())
                        }
                        Err(e) => RespType::RError(e.to_string()),
                    },
                    Err(e) => RespType::RError(e.to_string()),
                };
            }
        }
    }
    RespType::RError(String::from("Invalid command decrby"))
}

pub fn incrby(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 2 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            if let RespType::RBulkString(incr) = &cmd[2] {
                let number = incr.parse::<i64>();
                return match number {
                    Ok(incr) => match db.increment_key_by(key, incr) {
                        Ok(res) => RespType::RInteger(res.try_into().unwrap()),
                        Err(e) => RespType::RError(e.to_string()),
                    },
                    Err(e) => RespType::RError(e.to_string()),
                };
            }
        }
    }
    RespType::RError(String::from("Invalid command incrby"))
}

pub fn get(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            return match db.get_value_by_key(key) {
                Some(str) => RespType::RBulkString(str),
                None => {
                    //nil - testear
                    RespType::RNullBulkString()
                }
            };
        }
    }
    RespType::RError(String::from("Invalid command get"))
}

pub fn mget(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut db = database.write().unwrap();
    let mut vec_keys_with_string_values = vec![];
    if cmd.len() > 1 {
        for n in cmd.iter().skip(1) {
            if let RespType::RBulkString(current_key) = n {
                if db.key_exists(current_key.to_string()) {
                    let actual_string_value =
                        RespType::RBulkString(db.get_value_by_key_or_nil(current_key).unwrap());
                    vec_keys_with_string_values.push(actual_string_value)
                }
            }
        }
        RespType::RArray(vec_keys_with_string_values)
    } else {
        RespType::RError(String::from("Invalid command get"))
    }
}

pub fn bajar_resptype_a_vec_string(cmd: &[RespType]) -> Vec<String> {
    let mut vec_aux = vec![];
    for elemento in cmd.iter() {
        if let RespType::RBulkString(current_elemento) = elemento {
            vec_aux.push(current_elemento.to_string());
        }
    }
    vec_aux
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
    RespType::RError(String::from("Invalid command getdel"))
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
    RespType::RError(String::from("Invalid command getset"))
}

pub fn strlen(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            return match db.get_strlen_by_key(key) {
                Some(len) => RespType::RInteger(len),
                None => RespType::RError(String::from("key must hold a value of type string")),
            };
        }
    }
    RespType::RError(String::from("Invalid command strlen"))
}

pub fn mset(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut db = database.write().unwrap();
    if cmd.len() % 2 == 1 {
        let mut vec_aux = vec![];
        for elemento in cmd.iter().skip(1) {
            if let RespType::RBulkString(current_elemento) = elemento {
                vec_aux.push(current_elemento.to_string());
            }
        }
        for (pos, e) in vec_aux.iter().enumerate().step_by(2) {
            let vt_item = ValueTimeItem::new_now(
                ValueType::StringType(vec_aux[pos + 1].to_string()),
                KeyAccessTime::Persistent,
            );
            db.add(e.to_string(), vt_item);
        }
        RespType::RBulkString("Ok".to_string())
    } else {
        RespType::RBulkString("One or more parameters are missing".to_string())
    }
}

pub fn set(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            if let RespType::RBulkString(value) = &cmd[2] {
                let options = generate_options(cmd);
                let mut db = database.write().unwrap();
                let timeout = (&options[0].0.to_owned(), options[0].1);
                if db.set_string(key, value, timeout, options[1].1, options[2].1) {
                    return RespType::RBulkString(String::from("Ok"));
                } else {
                    return RespType::RNullBulkString();
                }
            }
        }
    }
    RespType::RNullBulkString()
}

fn generate_options(cmd: &[RespType]) -> Vec<(String, Option<&String>)> {
    let mut options = vec![
        (String::from("expire_at"), None),
        (String::from("set_if_exists"), None),
        (String::from("get_old_value"), None),
    ];
    for (pos, argumento) in cmd.iter().skip(3).enumerate() {
        if let RespType::RBulkString(arg) = argumento {
            if (arg == "ex") || (arg == "px") || (arg == "exat") || (arg == "pxat")
            // || (arg == "keepttl") -> no entiendo que hace
            {
                if let RespType::RBulkString(expire_at) = &cmd[pos + 1] {
                    options[0].0 = arg.to_string();
                    options[0].1 = Some(expire_at);
                }
            } else if arg == "xx" || arg == "nx" {
                options[1].1 = Some(arg);
            } else if arg == "get" {
                options[2].1 = Some(arg);
            }
        }
    }
    options
}
