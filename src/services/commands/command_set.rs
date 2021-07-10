use crate::domain::entities::key_value_item::{KeyAccessTime, ValueTimeItem, ValueType};
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
/// use proyecto_taller_1::services::utils::resp_type::RespType;use proyecto_taller_1::services::commands::command_set;
/// use proyecto_taller_1::domain::implementations::database::Database;
/// use std::sync::{Arc, RwLock};
/// use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
/// use std::collections::HashSet;
///
/// let db = Database::new("dummy_db_doc_set1.csv".to_string());
/// let mut database = Arc::new(RwLock::new(db));
/// let mut set = HashSet::new();
/// set.insert("kiwi".to_string());
/// set.insert("pomelo".to_string());
/// set.insert("sandia".to_string());
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItem::new_now(
/// ValueType::SetType(set),
/// KeyAccessTime::Persistent
/// ));
///
/// let res = command_set::add(&vec![
/// RespType::RBulkString("SADD".to_string()),
/// RespType::RBulkString("frutas".to_string()),
/// RespType::RBulkString("frutillas".to_string()),
/// ], &database);
///
/// match res {
/// RespType::RInteger(qty) => {
/// assert_eq!(qty,1)
///}
/// _ => assert!(false)
/// }
/// let _ = std::fs::remove_file("dummy_db_doc_set1.csv");
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
                        let vti = ValueTimeItem::new_now(
                            ValueType::SetType(set),
                            KeyAccessTime::Persistent,
                        );
                        db.add(key.to_string(), vti);
                        RespType::RInteger(1)
                    }
                    Some(value_item) => {
                        if let ValueType::SetType(mut old_value) = value_item.get_copy_of_value() {
                            let res = old_value.insert(value_to_add.to_string());
                            value_item._set_value(ValueType::SetType(old_value));
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
            String::from("Invalid command sadd")
        }
    }
    RespType::RError(String::from("Invalid command sadd"))
}

pub fn scard(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            return RespType::RInteger(db.get_len_of_set(key));
        }
    }
    RespType::RInteger(0)
}

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
