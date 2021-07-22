//! Servicio que implementa todos los comandos de tipo Key

use crate::domain::entities::key_value_item::KeyAccessTime;
use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// Elimina las claves recibidas en el comando.
///
/// Si alguna clave no existe, es ignorada.
/// Devuelve la cantidad de claves eliminadas.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
///
/// # let db = Database::new("dummy_db_del.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("pera")),
///     KeyAccessTime::Persistent
/// ));
/// database.write().unwrap().add("verdura".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("lechuga")),
///     KeyAccessTime::Persistent
/// ));
/// database.write().unwrap().add("postre".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("helado")),
///     KeyAccessTime::Persistent
/// ));
///
/// let res = command_key::del(&vec![
///     RespType::RBulkString("DEL".to_string()),
///     RespType::RBulkString("fruta".to_string()),
///     RespType::RBulkString("verdura".to_string()),
///     ], &database);
///
/// # match res {
/// #    RespType::RInteger(removed) => {
///         assert_eq!(removed, 2)
/// #    }
/// #    _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_del.csv");
/// ```
pub fn del(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut n_key_deleted = 0;
    for n in cmd.iter().skip(1) {
        if let RespType::RBulkString(current_key) = n {
            let mut new_database = database.write().unwrap();
            if new_database.delete_key(current_key.to_string()) {
                n_key_deleted += 1;
            }
        }
    }
    RespType::RInteger(n_key_deleted)
}

/// Copia el valor almacenado en la clave `source` en la clave `destination`.
///
/// Si el comando contiene el parametro "replace", en el caso de que ya exista una clave con el mismo nombre que la clave destino,
/// se reemplaza su valor por el valor de la clave fuente y se devuelve un entero 1.
/// En el caso que la clave destino ya exista pero no se incluya el parametro "replace", entonces no se copia el valor y se devuelve un entero 0.
/// Si no existe la clave, se crea, se guarda una copia del valor que guarda la clave fuente y se devuelve un entero 1.
/// Ante algun error se devuelve un entero 0.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
///
/// # let db = Database::new("dummy_db_copy.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("dolly".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("sheep")),
///     KeyAccessTime::Persistent
/// ));
///
/// let res = command_key::copy(&vec![
///     RespType::RBulkString("COPY".to_string()),
///     RespType::RBulkString("dolly".to_string()),
///     RespType::RBulkString("clone".to_string()),
///     ], &database);
///
/// # match res {
/// #    RespType::RInteger(copied) => {
///         assert_eq!(copied, 1)
/// #    }
/// #    _ => assert!(false)
/// # }
///
/// database.write().unwrap().add("pet".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("cat")),
///     KeyAccessTime::Persistent
/// ));
///
/// let res = command_key::copy(&vec![
///     RespType::RBulkString("COPY".to_string()),
///     RespType::RBulkString("pet".to_string()),
///     RespType::RBulkString("clone".to_string()),
///     RespType::RBulkString("replace".to_string()),
///     ], &database);
///
/// # match res {
/// #    RespType::RInteger(copied) => {
///         assert_eq!(copied, 1)
/// #    }
/// #    _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_copy.csv");
/// ```
pub fn copy(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 2 {
        if let RespType::RBulkString(source) = &cmd[1] {
            if let RespType::RBulkString(destination) = &cmd[2] {
                if let Ok(write_guard) = database.write() {
                    let mut db = write_guard;
                    let replace = copy_should_replace(cmd);
                    match db.copy(source.to_string(), destination.to_string(), replace) {
                        Some(_) => {
                            return RespType::RInteger(1);
                        }
                        None => {
                            return RespType::RInteger(0);
                        }
                    }
                }
            }
        }
    }
    RespType::RInteger(0)
}

/// Verifica que si el ultimo parametro es `replace`.
/// Si es `replace`, devuelve true, sino devuelve false.
fn copy_should_replace(cmd: &[RespType]) -> bool {
    if cmd.len() == 4 {
        if let RespType::RBulkString(replace) = &cmd[3] {
            if replace == "replace" {
                return true;
            }
        }
    }
    false
}

/// Devuelve si las claves especificadas existe.
///
/// Si se repite la misma clave, se va a contar repetidas veces. Por ejemplo, el comando `EXISTS key key` devuelve 2.
/// Devuelve la cantidad de claves existentes.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
///
/// # let db = Database::new("dummy_db_exists.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("pera")),
///     KeyAccessTime::Persistent
/// ));
/// database.write().unwrap().add("verdura".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("lechuga")),
///     KeyAccessTime::Persistent
/// ));
///
/// let res = command_key::exists(&vec![
///     RespType::RBulkString("EXISTS".to_string()),    
///     RespType::RBulkString("fruta".to_string()),
///     RespType::RBulkString("verdura".to_string()),
///     ], &database);
///
/// # match res {
/// #    RespType::RInteger(keys_count) => {
///         assert_eq!(keys_count, 2)
/// #    }
/// #    _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_exists.csv");
/// ```
pub fn exists(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut key_found = 0;
    if cmd.len() > 1 {
        for n in cmd.iter().skip(1) {
            if let RespType::RBulkString(current_key) = n {
                if database
                    .write()
                    .unwrap()
                    .key_exists(current_key.to_string())
                {
                    key_found += 1;
                }
            }
        }
    }
    RespType::RInteger(key_found)
}

/// Cambia el tipo de clave de volatil a persistente.
///
/// Elimina el timeout de la clave, convirtiendola en una clave persistente.
/// Si la clave no existe o no tiene un timeout asociado, devuelve un 0.
/// En caso de exito devuelve un 1.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
///
/// # let db = Database::new("dummy_db_persist.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("pera")),
///     KeyAccessTime::Volatile(1925487534)
/// ));
///
/// let res = command_key::persist(&vec![
///     RespType::RBulkString("PERSIST".to_string()),
///     RespType::RBulkString("fruta".to_string()),
///     ], &database);
///
/// # match res {
/// #    RespType::RInteger(persisted) => {
///         assert_eq!(persisted, 1)
/// #    }
/// #    _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_persist.csv");
/// ```
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

/// Renombra una clave.
///
/// Si la clave no existe, devuelve error.
/// Si el nuevo nombre de la clave ya existe, la sobreescribe.
/// En caso de exito, devuelve "OK".
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
///
/// # let db = Database::new("dummy_db_rename.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("animal".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("perro")),
///     KeyAccessTime::Persistent
/// ));
///
/// let res = command_key::rename(&vec![
///     RespType::RBulkString("RENAME".to_string()),
///     RespType::RBulkString("animal".to_string()),
///     RespType::RBulkString("mascota".to_string()),
///     ], &database);
///
/// # match res {
/// #    RespType::RBulkString(response) => {
///         assert_eq!(response, "OK".to_string())
/// #    }
/// #    _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_rename.csv");
/// ```
pub fn rename(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(current_key) = &cmd[1] {
            let mut new_database = database.write().unwrap();
            if let RespType::RBulkString(new_key) = &cmd[2] {
                new_database.rename_key(current_key.to_string(), new_key.to_string());
            }
        }
    }
    RespType::RBulkString("OK".to_string())
}

/// Configura un tiempo de expiracion sobre una clave a partir del momento en que se envia el comando.
///
/// Configura un tiempo de expiracion sobre una clave (la clave se dice que
/// es volatil). Luego de ese tiempo de expiracion, la clave es automaticamente eliminada.
/// El comando recibe 2 parámetros: la key y el tiempo de expiración (en segundos)
/// Devuelve 1 si pudo ser configurado, o 0 en caso contrario.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
///
/// # let db = Database::new("dummy_db_expire.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("pera")),
///     KeyAccessTime::Persistent
/// ));
///
/// let res = command_key::expire(&vec![
///     RespType::RBulkString("EXPIRE".to_string()),
///     RespType::RBulkString("fruta".to_string()),
///     RespType::RBulkString("30".to_string()),
///     ], &database);
///
/// # match res {
/// #    RespType::RInteger(expire) => {
///         assert_eq!(expire, 1)
/// #    }
/// #    _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_expire.csv");
/// ```
pub fn expire(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() != 3 {
    } else if let RespType::RBulkString(key) = &cmd[1] {
        let mut db = database.write().unwrap();
        if let RespType::RBulkString(timeout) = &cmd[2] {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            let new_time = u64::from_str(timeout).unwrap() + now.as_secs();
            let result = db.expire_key(key, &new_time.to_string());
            return if result {
                RespType::RInteger(1)
            } else {
                RespType::RInteger(0)
            };
        }
    }
    RespType::RInteger(0)
}
/// Configura un tiempo de expiracion UNIX sobre una clave.
///
/// Configura un tiempo de expiracion sobre una clave (la clave se dice que
/// es volatil). Luego de ese tiempo de expiracion, la clave es automaticamente eliminada.
/// El comando recibe 2 parámetros: la key y el tiempo de expiración (en timestamp UNIX)
/// Devuelve 1 si pudo ser configurado, o 0 en caso contrario.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
///
/// # let db = Database::new("dummy_db_expireat.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("pera")),
///     KeyAccessTime::Persistent
/// ));
///
/// let res = command_key::expireat(&vec![
///     RespType::RBulkString("EXPIREAT".to_string()),
///     RespType::RBulkString("fruta".to_string()),
///     RespType::RBulkString("1925487534".to_string()),
///     ], &database);
///
/// # match res {
/// #    RespType::RInteger(expire) => {
///         assert_eq!(expire, 1)
/// #    }
/// #    _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_expireat.csv");
/// ```
pub fn expireat(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() != 3 {
    } else if let RespType::RBulkString(key) = &cmd[1] {
        let mut db = database.write().unwrap();
        if let RespType::RBulkString(timeout) = &cmd[2] {
            let result = db.expire_key(key, timeout);
            return if result {
                RespType::RInteger(1)
            } else {
                RespType::RInteger(0)
            };
        }
    }
    RespType::RInteger(0)
}

/// Devuelve los elementos contenidos en una lista o set de forma ordenada.
///
/// Ordena una lista o set alojado en `key`.Por defecto, ordena de mayor a menos.
/// Admite los parámetros:
/// * DESC: Ordena de mayor a menor.
/// * ALPHA: Ordena alfabeticamente.
/// * LIMIT lower count: Limita la cantidad de elementos. Toma `count` elementos desde la posicion `lower`.
/// * BY: Permite ordenar a partir de claves externas y sus valores asociados.
/// * STORE key: Almacena la lista ordenada en `key`.
///
/// Devuelve una lista con los elementos ordenados. Si se especificó el parámetro `store`, devuelve la cantidad de elementos ordenados y almacenados en la nueva clave.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
///
/// # let db = Database::new("dummy_db_sort.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItem::new_now(
///     ValueType::ListType(vec![String::from("pera"), String::from("manzana"), String::from("sandia")]),
///     KeyAccessTime::Persistent
/// ));
///
/// let res = command_key::sort(&vec![
///     RespType::RBulkString("SORT".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     ], &database);
///
/// # match res {
/// #    RespType::RArray(sorted) => {
///         assert_eq!(sorted, vec![RespType::RBulkString("manzana".to_string()), RespType::RBulkString("pera".to_string()), RespType::RBulkString("sandia".to_string())])
/// #    }
/// #    _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_sort.csv");
/// ```
pub fn sort(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    //A hashpam is created to store all info about the SORT operation
    let aux_hash_map = generate_hashmap(cmd);
    let mut vector = Vec::new();
    if let RespType::RBulkString(current_key) = &cmd[1] {
        let mut sorted_list: Vec<&String> = Vec::new();
        let mut auxiliary_vec = Vec::new();
        let mut database_lock = database.write().unwrap();
        if aux_hash_map.contains_key("by") {
            if let RespType::RBulkString(pat) = aux_hash_map.get("by").unwrap() {
                let mut tuple_vector = database_lock
                    .get_values_and_associated_external_key_values(
                        pat.to_string(),
                        current_key.to_string(),
                    )
                    .unwrap();
                tuple_vector.sort_by_key(|k| k.1.clone());
                for val in tuple_vector {
                    auxiliary_vec.push(val.0.clone());
                }
                for j in &auxiliary_vec {
                    sorted_list.push(j)
                }
                if aux_hash_map.contains_key("desc") {
                    sorted_list.reverse()
                }
                if (aux_hash_map.contains_key("lower")) || (aux_hash_map.contains_key("upper")) {
                    if let RespType::RBulkString(lower_bound) = aux_hash_map.get("lower").unwrap() {
                        if let RespType::RBulkString(upper_bound) =
                            aux_hash_map.get("upper").unwrap()
                        {
                            let min = lower_bound.parse::<usize>().unwrap();
                            let max = upper_bound.parse::<usize>().unwrap();
                            sorted_list = sorted_list[min..max].to_vec();
                        }
                    }
                }
            }
        } else {
            let my_list_value_optional = database_lock.get_live_item(current_key);
            if let Some(my_list_value) = my_list_value_optional {
                if aux_hash_map.contains_key("desc") {
                    //ordeno descendentemente
                    sorted_list = my_list_value.sort_descending().unwrap();
                } else {
                    //ordeno ascendentemente
                    sorted_list = my_list_value.sort().unwrap();
                }
                if (aux_hash_map.contains_key("lower")) || (aux_hash_map.contains_key("upper")) {
                    if let RespType::RBulkString(lower_bound) = aux_hash_map.get("lower").unwrap() {
                        if let RespType::RBulkString(upper_bound) =
                            aux_hash_map.get("upper").unwrap()
                        {
                            let min = lower_bound.parse::<usize>().unwrap();
                            let max = upper_bound.parse::<usize>().unwrap();
                            sorted_list = sorted_list[min..max].to_vec();
                            //sorted_list = sort_vec_by_min_max_values(lower_bound, upper_, sorted_list);
                        }
                    }
                }
            }
        }
        sorted_list
            .into_iter()
            .for_each(|value| vector.push(RespType::RBulkString(value.to_string())));
        RespType::RArray(vector)
    } else {
        RespType::RBulkString("empty".to_string())
    }
}

/// Devuelve todas las claves que coinciden con el patrón especificado.
///
/// El patrón debe ser glob-style, por ejemplo:
///
/// h\?llo coincide con hello, hallo and hxllo
///
/// h\*llo coincide con hllo and heeeello
///
/// h\[ae\]llo coincide con hello and hallo, pero no con hillo
///
/// h\[\^e\]llo coincide con hallo, hbllo, ... pero no con hello
///
/// h\[a-b\]llo coincide con hallo and hbllo
///
/// Devuelve una lista con los claves que coinciden con el patrón.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
///
/// # let db = Database::new("dummy_db_keys.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
///  database.write().unwrap().add("animal".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("oso")),
///     KeyAccessTime::Persistent
///  ));
///  database.write().unwrap().add("animacion".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("soul")),
///     KeyAccessTime::Persistent
///  ));
///  database.write().unwrap().add("comida".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("pizza")),
///     KeyAccessTime::Persistent
///  ));
///
/// let res = command_key::keys(&vec![
///     RespType::RBulkString("KEYS".to_string()),
///     RespType::RBulkString("anima*".to_string()),
///     ], &database);
///
/// # match res {
/// #    RespType::RArray(matched) => {
///         assert!(matched.contains(&RespType::RBulkString("animal".to_string())));
///         assert!(matched.contains(&RespType::RBulkString("animacion".to_string())));
/// #    }
/// #    _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_keys.csv");
/// ```
pub fn keys(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if let RespType::RBulkString(pattern) = &cmd[1] {
        let new_database = database.read().unwrap();
        let pattern_matching_keys = new_database.get_keys_that_match_pattern(pattern);
        let mut vec = vec![];
        pattern_matching_keys
            .into_iter()
            .for_each(|value| vec.push(RespType::RBulkString(value)));
        RespType::RArray(vec)
    } else {
        RespType::RBulkString("No matching keys".to_string())
    }
}
/// Actualiza el `last_access_time` de las keys recibidas.
///
/// A partir de una lista de `keys` enviadas, se encarga de actualizar con now
/// el `last_access_time`de la key. Si la key no existe o expiró se ignora.
/// Devuelve la cantidad de keys actualizadas.
///
/// # Ejemplos
///
/// 1. Actualiza dos `keys` válidas:
///
/// ```
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, ValueTimeItem, KeyAccessTime};
/// # use std::time::SystemTime;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
///
/// // Agrego los datos en la base
///
/// # let db = Database::new("dummy_db_doc_touch1.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// let mut timeout_10seg = SystemTime::now()
///  .duration_since(SystemTime::UNIX_EPOCH)
///   .unwrap().as_secs();
/// timeout_10seg += 10;
///
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItem::new_now(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()]),
///     KeyAccessTime::Persistent
/// ));
///
/// database.write().unwrap().add("verduras".to_string(),ValueTimeItem::new_now(
///     ValueType::ListType(vec!["acelga".to_string(),"cebolla".to_string(),"zanahoria".to_string()]),
///     KeyAccessTime::Volatile(timeout_10seg)
/// ));
///
/// //Ejecuto el comando con los parámetros necesarios:
/// let res = command_key::touch(&vec![
///     RespType::RBulkString("TOUCH".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("verduras".to_string())
/// ], &database);
///
/// # match res {
/// #     RespType::RInteger(quantity) => {
///     assert_eq!(quantity, 2)
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # let _ = std::fs::remove_file("dummy_db_doc_touch1.csv");
/// ```
/// 2. Itenta actualizar 2 `keys` donde una está expirada y la otra no existe en la database
/// ```
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, ValueTimeItem, KeyAccessTime};
/// # use std::time::{SystemTime, Duration};
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use std::thread::sleep;
///
/// # let db = Database::new("dummy_db_doc_touch2.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// let timeout_now = SystemTime::now()
///  .duration_since(SystemTime::UNIX_EPOCH)
///   .unwrap().as_secs();
///
/// sleep(Duration::from_secs(1));
///
/// database.write().unwrap().add("verduras".to_string(),ValueTimeItem::new_now(
///     ValueType::ListType(vec!["acelga".to_string(),"cebolla".to_string(),"zanahoria".to_string()]),
///     KeyAccessTime::Volatile(timeout_now)
/// ));
///
/// //Ejecuto el comando con los parámetros necesarios:
/// let res = command_key::touch(&vec![
///     RespType::RBulkString("TOUCH".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("verduras".to_string())
/// ], &database);
///
/// # match res {
/// #    RespType::RInteger(quantity) => {
/// #    assert_eq!(quantity, 0)
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # let _ = std::fs::remove_file("dummy_db_doc_touch2.csv");
/// ```
pub fn touch(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut number_of_touched_keys = 0;
    let mut db = database.write().unwrap();
    if cmd.len() > 1 {
        for n in cmd.iter().skip(1) {
            if let RespType::RBulkString(current_key) = n {
                if db.reboot_time(current_key.to_string()).is_some() {
                    number_of_touched_keys += 1
                }
            }
        }
    }
    RespType::RInteger(number_of_touched_keys)
}

/// Retorna el tiempo que le queda a una clave para que se cumpla su timeout (en segundos).
///
/// En caso que no sea una clave volátil retorna -1. Si no existe la clave, retorna -2.
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
///
/// # let db = Database::new("dummy_db_ttl.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("pera")),
///     KeyAccessTime::Volatile(1925487534)
/// ));
///
/// let res = command_key::get_ttl(&vec![
///     RespType::RBulkString("TTL".to_string()),
///     RespType::RBulkString("fruta".to_string())
///     ], &database);
///
/// # match res {
/// #    RespType::RInteger(time) => {
///         assert!(time > 0)
/// #    }
/// #    _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_ttl.csv");
/// ```
pub fn get_ttl(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            return match db.get_live_item(key) {
                None => RespType::RSignedNumber(-2),
                Some(item) => match item.get_timeout() {
                    KeyAccessTime::Volatile(timeout) => RespType::RInteger(*timeout as usize),
                    KeyAccessTime::Persistent => RespType::RInteger(0),
                },
            };
        }
    }
    RespType::RInteger(0)
}

/// Retorna el tipo de dato almacenado en `key`.
///
/// Los tipos de datos posibles son: string, list y set.
/// Si la clave no existe, devuelve none.
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
///
/// # let db = Database::new("dummy_db_type.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItem::new_now(
///     ValueType::StringType(String::from("pera")),
///     KeyAccessTime::Persistent
/// ));
///
/// let res = command_key::get_type(&vec![
///     RespType::RBulkString("TYPE".to_string()),
///     RespType::RBulkString("fruta".to_string())
///     ], &database);
///
/// # match res {
/// #    RespType::RBulkString(value_type) => {
///         assert_eq!(value_type, "string".to_string())
/// #    }
/// #    _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_type.csv");
/// ```
pub fn get_type(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut tipo = String::from("");
    if let RespType::RBulkString(current_key) = &cmd[1] {
        if database
            .write()
            .unwrap()
            .key_exists(current_key.to_string())
        {
            tipo = database
                .read()
                .unwrap()
                .get_type_of_value(current_key.to_string());
        }
    }
    RespType::RBulkString(tipo)
}

fn _sort_vec_by_min_max_values(
    lower_bound: &str,
    upper_bound: &str,
    sorted_list: Vec<&String>,
) -> Vec<String> {
    let min = lower_bound.parse::<usize>().unwrap();
    let max = upper_bound.parse::<usize>().unwrap();
    let list = sorted_list[min..max].to_vec();
    let mut aux = vec![];
    for elemento in list {
        aux.push(elemento.to_string());
    }
    aux
}

/// Permite generar un hashmap a partir de un grupo de claves hardcodeadas y asociarles un valor de existencia
fn generate_hashmap(cmd: &[RespType]) -> HashMap<String, &RespType> {
    let mut aux_hash_map = HashMap::new();
    let mut posicion = 1;
    for argumento in cmd.iter().skip(1) {
        if let RespType::RBulkString(arg) = argumento {
            if (arg == "asc") || (arg == "desc") || (arg == "alpha") {
                aux_hash_map.insert(arg.to_string(), &RespType::RInteger(1));
            } else if (arg == "by") || (arg == "store") {
                aux_hash_map.insert(arg.to_string(), &cmd[posicion + 1]);
            } else if arg == "limit" {
                aux_hash_map.insert("lower".to_string(), &cmd[posicion + 1]);
                aux_hash_map.insert("upper".to_string(), &cmd[posicion + 2]);
            } else {
                aux_hash_map.insert("key".to_string(), argumento);
            }
        }
        posicion += 1;
    }
    aux_hash_map
}

#[test]
fn test_001_se_genera_un_hashmap_a_partir_de_vector_con_asc() {
    let operation = vec![RespType::RBulkString("ASC".to_string())];
    let hm = generate_hashmap(&operation);
    for (key, value) in hm {
        println!("{:?}: {:?}", key, value)
    }
}

#[test]
fn test_002_se_genera_un_hashmap_a_partir_de_vector_con_by_algun_valor() {
    let operation = vec![
        RespType::RBulkString("BY".to_string()),
        RespType::RBulkString("algun_valor".to_string()),
    ];
    let hm = generate_hashmap(&operation);
    for (key, value) in hm {
        println!("{:?}: {:?}", key, value)
    }
}

#[test]
fn test_003_se_genera_un_hashmap_a_partir_de_vector_con_limit_y_los_extremos() {
    let operation = vec![
        RespType::RBulkString("LIMIT".to_string()),
        RespType::RBulkString("0".to_string()),
        RespType::RBulkString("10".to_string()),
    ];
    let hm = generate_hashmap(&operation);
    for (key, value) in hm {
        println!("{:?}: {:?}", key, value)
    }
}
