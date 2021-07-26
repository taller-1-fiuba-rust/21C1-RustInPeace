//! Servicio que implementa todos los comandos de tipo Set

use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
use std::sync::{Arc, RwLock};

/// Agrega un elemento al set de la `key` dada
///
/// A partir de una `key` dada, se busca el set asociado y se le agrega el string que se
/// pase por parámetro.
/// Si la `key` no existe, se crea un SET nuevo el valor enviado.
/// Si el valor almacenado en la `key` no es un SET, retorna error.
///
/// Devuelve la cantidad de valores que se agregaron al SET.
///
/// # Ejemplos
///
/// 1. Se agregan dos valores a una `key`
///
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_set;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem, ValueTimeItemBuilder};
/// # use std::collections::HashSet;
///
/// # let db = Database::new("dummy_db_add.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// let mut set = HashSet::new();
/// set.insert("kiwi".to_string());
/// set.insert("pomelo".to_string());
/// set.insert("sandia".to_string());
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::SetType(set)
/// ).build());
///
/// let res = command_set::add(&vec![
///     RespType::RBulkString("SADD".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("frutillas".to_string()),
/// ], &database);
///
/// # match res {
/// # RespType::RInteger(quantity) => {
/// assert_eq!(quantity,1)
/// # }
/// # _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_add.csv");
/// ```
pub fn add(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 2 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database
                .write()
                .expect("Could not get database lock on add");
            let mut values_to_add = Vec::new();
            for n in cmd.iter().skip(2) {
                if let RespType::RBulkString(value) = n {
                    values_to_add.push(value);
                }
            }
            match db.add_element_to_set(key, values_to_add) {
                Some(added) => {
                    return RespType::RInteger(added);
                }
                None => {
                    return RespType::RError(String::from("Value stored should be a set"));
                }
            }
        }
    }
    RespType::RError(String::from("Invalid command sadd"))
}

/// Retorna la cantidad de elementos del SET almacenado en `key`.
///
/// Si la `key` no existe o el valor almacenado en la `key` no es un SET, retorna 0.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_set;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem, ValueTimeItemBuilder};
/// # use std::collections::HashSet;
///
/// # let db = Database::new("dummy_db_scard.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// let mut set = HashSet::new();
/// set.insert("kiwi".to_string());
/// set.insert("pomelo".to_string());
/// set.insert("sandia".to_string());
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::SetType(set)).build());
///
/// let res = command_set::scard(&vec![
///     RespType::RBulkString("SCARD".to_string()),
///     RespType::RBulkString("frutas".to_string())],
///     &database);
///
/// assert_eq!(res, RespType::RInteger(3));
/// # let _ = std::fs::remove_file("dummy_db_scard.csv");
/// ```
pub fn scard(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let db = database
                .read()
                .expect("Could not get database read lock on scard");
            let (item, expired) = db.check_timeout_item(key);
            if item.is_some() && expired {
                drop(db);
                database
                    .write()
                    .expect("Could not get database write lock on scard")
                    .remove_expired_key(key)
            }
            let db = database
                .read()
                .expect("Could not get database read lock on scard");
            return RespType::RInteger(db.get_len_of_set(key));
        }
    }
    RespType::RInteger(0)
}

/// Retorna si el elemento pertenece al SET almacenado en la clave especificada.
///
/// Si el elemento pertenece al SET, retorna 1.
/// Si la clave no existe o el valor almacenado no es un SET, retorna 0.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_set;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem, ValueTimeItemBuilder};
/// # use std::collections::HashSet;
///
/// # let db = Database::new("dummy_db_sismember.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// let mut set = HashSet::new();
/// set.insert("kiwi".to_string());
/// set.insert("pomelo".to_string());
/// set.insert("sandia".to_string());
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::SetType(set)
/// ).build());
///
/// let res = command_set::sismember(&vec![
///     RespType::RBulkString("SISMEMBER".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("pomelo".to_string())],
///     &database);
///
/// assert_eq!(res, RespType::RInteger(1));
/// # let _ = std::fs::remove_file("dummy_db_sismember.csv");
/// ```
pub fn sismember(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 2 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let db = database
                .read()
                .expect("Could not get database read lock on smembers");
            let (item, expired) = db.check_timeout_item(key);
            if item.is_some() && expired {
                drop(db);
                database
                    .write()
                    .expect("Could not get database write lock on smembers")
                    .remove_expired_key(key)
            }
            if let RespType::RBulkString(member) = &cmd[2] {
                let db = database
                    .read()
                    .expect("Could not get database read lock on smembers");
                return RespType::RInteger(db.is_member_of_set(key, member));
            }
        }
    }
    RespType::RInteger(0)
}

/// Retorna todos los elementos pertenecientes al SET almacenado en la clave especificada.
///
/// Devuelve un array con todos los elementos que pertenecen al SET.
/// Si la clave no existe, o si la clave no almacena un valor de tipo SET, devuelve un array nulo.
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_set;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem, ValueTimeItemBuilder};
/// # use std::collections::HashSet;
///
/// # let db = Database::new("dummy_db_smembers.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// let mut set = HashSet::new();
/// set.insert("kiwi".to_string());
/// set.insert("pomelo".to_string());
/// set.insert("sandia".to_string());
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::SetType(set)
/// ).build());
///
/// let res = command_set::smembers(&vec![
///     RespType::RBulkString("SMEMBERS".to_string()),
///     RespType::RBulkString("frutas".to_string())],
///     &database);
///
/// # match res {
/// # RespType::RArray(array) => {
/// assert!(array.contains(&RespType::RBulkString("kiwi".to_string())));
/// assert!(array.contains(&RespType::RBulkString("pomelo".to_string())));
/// assert!(array.contains(&RespType::RBulkString("sandia".to_string())));
///# }
/// # _ => assert!(false)
/// # }
///
/// # let _ = std::fs::remove_file("dummy_db_smembers.csv");
/// ```
pub fn smembers(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let db = database
                .read()
                .expect("Could not get database read lock on smembers");
            let (item, expired) = db.check_timeout_item(key);
            if item.is_some() && expired {
                drop(db);
                database
                    .write()
                    .expect("Could not get database write lock on smembers")
                    .remove_expired_key(key)
            }
            let db = database
                .read()
                .expect("Could not get database read lock on smembers");
            let members = db.get_members_of_set(key);
            return RespType::RArray(
                members
                    .iter()
                    .map(|member| RespType::RBulkString(member.to_string()))
                    .collect(),
            );
        }
    }
    RespType::RNullArray()
}

/// Elimina los elementos especificados del SET almacenado en `key`.
///
/// Retorna la cantidad de elementos eliminados del SET.
/// Si algún elemento no pertenece al set, se ignora.
/// Si `key` no existe, retorna 0.
/// Si el valor almacenado en `key` no es de tipo SET, retorna Error.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_set;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, ValueTimeItemBuilder};
/// # use std::collections::HashSet;
///
/// # let db = Database::new("dummy_db_srem.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// let mut set = HashSet::new();
/// set.insert("kiwi".to_string());
/// set.insert("pomelo".to_string());
/// set.insert("sandia".to_string());
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::SetType(set)
/// ).build());
///
/// let res = command_set::srem(&vec![
///     RespType::RBulkString("SREM".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("sandia".to_string()),
///     RespType::RBulkString("pomelo".to_string())],
///     &database);
///
/// assert_eq!(res, RespType::RInteger(2));
/// # let _ = std::fs::remove_file("dummy_db_srem.csv");
/// ```
pub fn srem(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut deleted = 0;
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database
                .write()
                .expect("Could not get database lock on srem");
            for n in cmd.iter().skip(2) {
                if let RespType::RBulkString(member) = n {
                    let removed = db.remove_member_from_set(key, member);
                    match removed {
                        Some(rem) => {
                            if rem {
                                deleted += 1;
                            }
                        }
                        None => {
                            return RespType::RError(format!(
                                "Value stored at key {} is not a Set",
                                key
                            ));
                        }
                    }
                }
            }
        }
    }
    RespType::RInteger(deleted)
}
