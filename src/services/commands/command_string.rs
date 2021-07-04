use crate::domain::entities::key_value_item::KeyAccessTime;
use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};
use crate::{domain::implementations::database::Database, services::utils::resp_type::RespType};
use std::collections::HashMap;
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

pub fn bajar_resptype_a_vec_string(cmd: &[RespType]) -> Vec<String> {
    let mut vec_aux = vec![];
    for elemento in cmd.into_iter() {
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
    if cmd.len() % 2 == 1 {
        let mut vec_aux = vec![];
        for elemento in cmd.iter().skip(1) {
            if let RespType::RBulkString(current_elemento) = elemento {
                vec_aux.push(current_elemento.to_string());
            }
        }
        for (pos, e) in vec_aux.iter().enumerate().step_by(2) {
            let vt_item = ValueTimeItem {
                value: ValueType::StringType(vec_aux[pos + 1].to_string()),
                timeout: KeyAccessTime::Volatile(0),
            };
            db.add(e.to_string(), vt_item);
        }
        RespType::RBulkString("Ok".to_string())
    } else {
        RespType::RBulkString("One or more parameters are missing".to_string())
    }
}

pub fn set(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    //A hashpam is created to store all info about the SORT operation
    let aux_hash_map = generate_hashmap(cmd);
    let mut _new_database = database.write().unwrap();
    if let RespType::RBulkString(_current_key) = aux_hash_map.get("key").unwrap() {
        if let RespType::RBulkString(_current_value) = aux_hash_map.get("value").unwrap() {
            if aux_hash_map.contains_key("EX") {
                if let RespType::RBulkString(_time_to_set) = aux_hash_map.get("EX").unwrap() {
                    //set "key":"value" con EX valor desde el hashmap
                }
            } else if aux_hash_map.contains_key("PX") {
                if let RespType::RBulkString(_time_to_set) = aux_hash_map.get("PX").unwrap() {
                    //set "key":"value" con PX valor desde el hashmap
                }
            } else if aux_hash_map.contains_key("EXAT") {
                if let RespType::RBulkString(_time_to_set) = aux_hash_map.get("EXAT").unwrap() {
                    //set "key":"value" con EXAT valor desde el hashmap
                }
            } else if aux_hash_map.contains_key("PXAT") {
                if let RespType::RBulkString(_time_to_set) = aux_hash_map.get("PXAT").unwrap() {
                    //set "key":"value" con PXAT valor desde el hashmap
                }
            } else if aux_hash_map.contains_key("KEEPTTL") {
                if let RespType::RBulkString(_time_to_set) = aux_hash_map.get("KEEPTTL").unwrap() {
                    //set "key":"value" con KEEPTTL valor desde el hashmap
                }
            } else if aux_hash_map.contains_key("NX") {
                if let RespType::RBulkString(_time_to_set) = aux_hash_map.get("NX").unwrap() {
                    //set "key":"value" con NX valor desde el hashmap
                }
            } else if aux_hash_map.contains_key("XX") {
                if let RespType::RBulkString(_time_to_set) = aux_hash_map.get("XX").unwrap() {
                    //set "key":"value" con XX valor desde el hashmap
                }
            }
        }
    }
    RespType::RBulkString("Ok".to_string())
}

//--------------------------------------------------------------------
/// Permite generar un hashmap a partir de un grupo de claves hardcodeadas y asociarles un valor de existencia
fn generate_hashmap(cmd: &[RespType]) -> HashMap<String, &RespType> {
    let mut aux_hash_map = HashMap::new();
    let keys = vec![
        "SET", "EX", "PX", "EXAT", "PXAT", "KEEPTTL", "NX", "XX ", "GET",
    ];
    let mut current_position;
    for key in keys {
        current_position = cmd
            .iter()
            .position(|x| x == &RespType::RBulkString(key.to_string()));
        if current_position != None {
            if key == "SET" {
                aux_hash_map.insert("key".to_string(), &cmd[current_position.unwrap() + 1]);
                aux_hash_map.insert("value".to_string(), &cmd[current_position.unwrap() + 2]);
                aux_hash_map.insert(key.to_string(), &RespType::RInteger(1));
            } else if key == "EX"
                || key == "PX"
                || key == "EXAT"
                || key == "PXAT"
                || key == "KEEPTLL"
            {
                aux_hash_map.insert(key.to_string(), &cmd[current_position.unwrap() + 1]);
            } else if key == "NX" || key == "XX" {
                aux_hash_map.insert(key.to_string(), &cmd[current_position.unwrap() + 1]);
            } else {
                aux_hash_map.insert(key.to_string(), &cmd[current_position.unwrap() + 1]);
            }
        }
    }
    aux_hash_map
}

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


//VER LOS PARAMETROS QUE SE PASAN A VALUETIMEITEM.. TODO HA CAMBIADO
// #[test]
// fn test_001_agregan_key_values_multiplemente_ninguna_clave_agregada_ya_existente() {
//     let db = Database::new("filename_13".to_string());
//     let database = Arc::new(RwLock::new(db));
//     //se rellena la database
//     let vt_1 = ValueTimeItem {
//         value: ValueType::StringType("hola".to_string()),
//         last_access_time: KeyAccessTime::Volatile(0),
//     };
//     let vt_2 = ValueTimeItem {
//         value: ValueType::StringType("chau".to_string()),
//         last_access_time: KeyAccessTime::Volatile(0),
//     };
//     let vt_3 = ValueTimeItem {
//         value: ValueType::ListType(vec!["hola".to_string(), "chau".to_string()]),
//         last_access_time: KeyAccessTime::Volatile(0),
//     };

//     load_data_in_db(&database, "saludo".to_string(), vt_1);
//     load_data_in_db(&database, "despido".to_string(), vt_2);
//     load_data_in_db(&database, "sal_dep".to_string(), vt_3);

//     let operation = vec![
//         RespType::RBulkString("mset".to_string()),
//         RespType::RBulkString("clave_1".to_string()),
//         RespType::RBulkString("valor_1".to_string()),
//         RespType::RBulkString("clave_2".to_string()),
//         RespType::RBulkString("valor_2".to_string()),
//     ];

//     let cosa = mset(&operation, &database);
//     println!("{:?}", cosa);
//     println!("{:?}", get_database_size(&database));
// }

// #[test]
// fn test_002_no_se_agregan_key_values_multiplemente_porque_hay_un_valor_faltante() {
//     let db = Database::new("filename_13".to_string());
//     let database = Arc::new(RwLock::new(db));
//     //se rellena la database
//     let vt_1 = ValueTimeItem {
//         value: ValueType::StringType("hola".to_string()),
//         last_access_time: KeyAccessTime::Volatile(0),
//     };
//     let vt_2 = ValueTimeItem {
//         value: ValueType::StringType("chau".to_string()),
//         last_access_time: KeyAccessTime::Volatile(0),
//     };
//     let vt_3 = ValueTimeItem {
//         value: ValueType::ListType(vec!["hola".to_string(), "chau".to_string()]),
//         last_access_time: KeyAccessTime::Volatile(0),
//     };

//     load_data_in_db(&database, "saludo".to_string(), vt_1);
//     load_data_in_db(&database, "despido".to_string(), vt_2);
//     load_data_in_db(&database, "sal_dep".to_string(), vt_3);

//     let operation = vec![
//         RespType::RBulkString("mset".to_string()),
//         RespType::RBulkString("clave_1".to_string()),
//         RespType::RBulkString("clave_2".to_string()),
//         RespType::RBulkString("valor_2".to_string()),
//     ];

//     let cosa = mset(&operation, &database);
//     println!("{:?}", cosa);
//     println!("{:?}", get_database_size(&database));
// }

// #[test]
// fn test_003_agregan_key_values_multiplemente_una_clave_agregada_ya_existente() {
//     let db = Database::new("filename_13".to_string());
//     let database = Arc::new(RwLock::new(db));
//     //se rellena la database
//     let vt_1 = ValueTimeItem {
//         value: ValueType::StringType("hola".to_string()),
//         last_access_time: KeyAccessTime::Volatile(0),
//     };
//     let vt_2 = ValueTimeItem {
//         value: ValueType::StringType("chau".to_string()),
//         last_access_time: KeyAccessTime::Volatile(0),
//     };
//     let vt_3 = ValueTimeItem {
//         value: ValueType::ListType(vec!["hola".to_string(), "chau".to_string()]),
//         last_access_time: KeyAccessTime::Volatile(0),
//     };

//     load_data_in_db(&database, "saludo".to_string(), vt_1);
//     load_data_in_db(&database, "despido".to_string(), vt_2);
//     load_data_in_db(&database, "sal_dep".to_string(), vt_3);

//     let operation = vec![
//         RespType::RBulkString("mset".to_string()),
//         RespType::RBulkString("saludo".to_string()),
//         RespType::RBulkString("valor_1".to_string()),
//         RespType::RBulkString("clave_2".to_string()),
//         RespType::RBulkString("valor_2".to_string()),
//     ];

//     let cosa = mset(&operation, &database);
//     println!("{:?}", cosa);
//     println!("{:?}", get_database_size(&database));
// }

// #[test]
// fn test_004_se_genera_un_hashmap_a_partir_de_vector_con_ex_algun_valor() {
//     let operation = vec![
//         RespType::RBulkString("SET".to_string()),
//         RespType::RBulkString("my_key".to_string()),
//         RespType::RBulkString("my_value".to_string()),
//         RespType::RBulkString("EX".to_string()),
//         RespType::RBulkString("60".to_string()),
//     ];
//     let hm = generate_hashmap(&operation);
//     for (key, value) in hm {
//         println!("{:?}: {:?}", key, value)
//     }
// }
