use crate::domain::entities::key_value_item::KeyAccessTime;
use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};
use crate::{domain::implementations::database::Database, services::utils::resp_type::RespType};
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
                match number {
                    Ok(decr) => match db.decrement_key_by(key, decr) {
                        Ok(res) => {
                            //falla si el nro es negativo
                            return RespType::RInteger(res.try_into().unwrap());
                        }
                        Err(e) => {
                            return RespType::RError(e.to_string());
                        }
                    },
                    Err(e) => {
                        return RespType::RError(e.to_string());
                    }
                }
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
                match number {
                    Ok(incr) => match db.increment_key_by(key, incr) {
                        Ok(res) => {
                            return RespType::RInteger(res.try_into().unwrap());
                        }
                        Err(e) => {
                            return RespType::RError(e.to_string());
                        }
                    },
                    Err(e) => {
                        return RespType::RError(e.to_string());
                    }
                }
            }
        }
    }
    RespType::RError(String::from("Invalid command incrby"))
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

pub fn mget(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let db = database.read().unwrap();
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
        return RespType::RArray(vec_keys_with_string_values);
    } else {
        RespType::RError(String::from("Invalid command get"))
    }
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
            let db = database.read().unwrap();
            match db.get_strlen_by_key(key) {
                Some(len) => {
                    return RespType::RInteger(len);
                }
                None => {
                    return RespType::RError(String::from("key must hold a value of type string"));
                }
            }
        }
    }
    RespType::RError(String::from("Invalid command strlen"))
}

pub fn mset(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut db = database.write().unwrap();
    // if cmd.len()%2==1 {
    let mut vec_aux = vec![];
    for elemento in cmd.iter().skip(1) {
        if let RespType::RBulkString(current_elemento) = elemento {
            vec_aux.push(current_elemento.to_string());
        }
    }
    for (pos, e) in vec_aux.iter().enumerate().step_by(2) {
        let vt_item = ValueTimeItem {
            value: ValueType::StringType(vec_aux[pos + 1].to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        db.add(e.to_string(), vt_item);
    }
    //}
    RespType::RBulkString("Ok".to_string())
}
// pub fn probando (cmd: &[RespType], database: &Arc<RwLock<Database>>) -> Vec<String> {
//     let mut db = database.write().unwrap();
//     let mut vec_aux = vec![];
//     for elemento in cmd.iter() {
//         if let RespType::RBulkString(current_elemento) = elemento {
//             vec_aux.push(current_elemento.to_string());
//         }
//     }
//     // for (pos, e) in vec_aux.iter().enumerate().step_by(2) {
//     //     let vt_item = ValueTimeItem {
//     //         value: ValueType::StringType(vec_aux[pos+1].to_string()),
//     //         last_access_time: KeyAccessTime::Volatile(0),
//     //     };
//     //     db.add(e.to_string(), vt_item);

//     // }
//     vec_aux
// }

pub fn load_data_in_db(database: &Arc<RwLock<Database>>, key: String, value: ValueTimeItem) {
    if let Ok(write_guard) = database.write() {
        let mut db = write_guard;
        db.add(key, value)
    }
}

pub fn get_database_size(database: &Arc<RwLock<Database>>) -> usize {
    if let Ok(write_guard) = database.read() {
        write_guard.get_size()
    } else {
        0
    }
}

#[test]
fn test_001_agregan_key_values_multiplemente() {
    let db = Database::new("filename_13".to_string());
    let database = Arc::new(RwLock::new(db));
    //se rellena la database
    let vt_1 = ValueTimeItem {
        value: ValueType::StringType("hola".to_string()),
        last_access_time: KeyAccessTime::Volatile(0),
    };
    let vt_2 = ValueTimeItem {
        value: ValueType::StringType("chau".to_string()),
        last_access_time: KeyAccessTime::Volatile(0),
    };
    let vt_3 = ValueTimeItem {
        value: ValueType::ListType(vec!["hola".to_string(), "chau".to_string()]),
        last_access_time: KeyAccessTime::Volatile(0),
    };

    load_data_in_db(&database, "saludo".to_string(), vt_1);
    load_data_in_db(&database, "despido".to_string(), vt_2);
    load_data_in_db(&database, "sal_dep".to_string(), vt_3);

    let _operation = RespType::RArray(vec![
        RespType::RBulkString("mget".to_string()),
        RespType::RBulkString("amigo_1".to_string()),
        RespType::RBulkString("juan".to_string()),
        RespType::RBulkString("amigo_2".to_string()),
        RespType::RBulkString("diana".to_string()),
    ]);
}
