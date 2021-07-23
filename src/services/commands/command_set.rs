//! Servicio que implementa todos los comandos de tipo Set

use crate::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType};
use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
use std::collections::HashSet;
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
///
pub fn add(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 2 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            if let RespType::RBulkString(value_to_add) = &cmd[2] {
                return match db.get_mut_live_item(key) {
                    None => {
                        // Creo el set
                        let mut set = HashSet::new();
                        set.insert(value_to_add.to_string());
                        let vti = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
                        db.add(key.to_string(), vti);
                        RespType::RInteger(1)
                    }
                    Some(value_item) => {
                        if let ValueType::SetType(mut old_value) = value_item.get_copy_of_value() {
                            let res = old_value.insert(value_to_add.to_string());
                            value_item.set_value(ValueType::SetType(old_value));
                            return if res {
                                RespType::RInteger(1)
                            } else {
                                RespType::RInteger(0) // ya existía el valor
                            };
                        } else {
                            RespType::RError(String::from("Value stored should be a set."))
                        }
                    }
                };
            }
        } else {
            String::from("Invalid command sadd");
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
            let mut db = database.write().unwrap();
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
            let mut db = database.write().unwrap();
            if let RespType::RBulkString(member) = &cmd[2] {
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
            let mut final_members = Vec::new();
            let mut db = database.write().unwrap();
            let members = db.get_members_of_set(key);
            members
                .iter()
                .for_each(|member| final_members.push(RespType::RBulkString(member.to_string())));
            return RespType::RArray(final_members);
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
            let mut db = database.write().unwrap();
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
