use crate::services::utils::resp_type::RespType;
use std::sync::{Arc, RwLock};
use crate::domain::implementations::database::Database;
use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
use std::collections::HashSet;

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
///
/// let db = Database::new("dummy_db_doc_set1.csv".to_string());
/// let mut database = Arc::new(RwLock::new(db));
/// //database.write().unwrap().add("frutas".to_string(),ValueTimeItem::new_now(
/// //ValueType::Set(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()]),
/// //KeyAccessTime::Persistent
/// //));
///
/// let res = command_set::add(&vec![
/// RespType::RBulkString("SADD".to_string()),
/// RespType::RBulkString("frutas".to_string()),
/// RespType::RBulkString("pomelo".to_string())
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
                match db.get_live_item(key){
                    None => { // Creo el set
                        let mut set = HashSet::new();
                        set.insert(value_to_add.to_string());
                        let vti = ValueTimeItem::new_now(ValueType::SetType(set), KeyAccessTime::Persistent);
                        db.add(value_to_add.to_string(), vti);
                        RespType::RInteger(1);
                    },
                    Some(value_item) => {
                        return match value_item.get_value(){
                            ValueType::SetType(_map) => { |mut map: HashSet<String>|
                                map.insert(value_to_add.to_string());
                                RespType::RInteger(1) // podría ser cero si ya existia el valor
                            },
                            _ => RespType::RError(String::from("Value stored should be a set."))

                        }
                    }
                }
            }
        }else{
            RespType::RError(String::from("Invalid command sadd"));
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
