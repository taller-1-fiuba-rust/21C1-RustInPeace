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
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_del.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("pera"))
/// ).build());
/// database.write().unwrap().add("verdura".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("lechuga"))
/// ).build());
/// database.write().unwrap().add("postre".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("helado"))
/// ).build());
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
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_copy.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("dolly".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("sheep"))
/// ).build());
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
/// database.write().unwrap().add("pet".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("cat"))
/// ).build());
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
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
///
///
/// let replace = command_key::copy_should_replace(&vec![
///     RespType::RBulkString("COPY".to_string()),
///     RespType::RBulkString("pet".to_string()),
///     RespType::RBulkString("clone".to_string()),
///     RespType::RBulkString("replace".to_string()),
///     ]);
///
/// assert!(replace);
/// ```
pub fn copy_should_replace(cmd: &[RespType]) -> bool {
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
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_exists.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("pera"))
/// ).build());
/// database.write().unwrap().add("verdura".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("lechuga"))
/// ).build());
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
                let (exist, expired) = database
                    .read()
                    .unwrap()
                    .key_exists_expired(current_key.to_string());
                if exist {
                    if !expired {
                        key_found += 1;
                    } else {
                        database.write().unwrap().remove_expired_key(current_key)
                    }
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
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_persist.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("pera"))).with_timeout(1925487534).build());
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
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_rename.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("animal".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("perro"))
/// ).build());
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
                if new_database.rename_key(current_key.to_string(), new_key.to_string()) {
                    RespType::RBulkString("OK".to_string())
                } else {
                    RespType::RError("key not found".to_string())
                }
            } else {
                RespType::RBulkString("missing parameters".to_string())
            }
        } else {
            RespType::RBulkString("missing parameters".to_string())
        }
    } else {
        RespType::RBulkString("missing parameters".to_string())
    }
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
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_expire.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("pera"))
/// ).build());
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
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_expireat.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("pera"))
/// ).build());
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
/// Ordena una lista o set alojado en `key`. Por defecto, ordena de menor a mayor.
///
/// Admite los parámetros:
///
/// * ASC | DESC: Ordena de menor a mayor (asc) o de mayor a menor (desc).
/// * ALPHA: Ordena alfabeticamente.
/// * LIMIT lower count: Limita la cantidad de elementos. Toma `count` elementos desde la posicion `lower`.
/// Si alguno de los límites no puede representarse con un número entero positivo, se asignan como default 0 para límite inferior y el largo del vector para límite superior.
/// * BY pattern: Permite ordenar a partir de claves externas y sus valores asociados.
///
/// Devuelve una lista con los elementos ordenados. Si se especifica el parámetro `store`, devuelve la cantidad de elementos ordenados y almacenados en la nueva clave.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_sort.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec![String::from("pera"), String::from("manzana"), String::from("sandia")])
/// ).build());
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
    let parameters = generate_hashmap(cmd);
    if let RespType::RBulkString(key) = &cmd[1] {
        let db = database.read().unwrap();
        let mut sorted: Vec<String> = Vec::new();
        if parameters.contains_key("by") {
            if let RespType::RBulkString(pattern) = parameters.get("by").unwrap() {
                let (mut elements_to_sort, expired) =
                    db.get_values_of_keys_matching_pattern(pattern.to_string(), key.to_string());
                for v in &expired {
                    database.write().unwrap().remove_expired_key(v)
                }

                elements_to_sort.sort_by_key(|k| k.1.to_owned());
                sorted = elements_to_sort.iter().map(|e| e.0.to_owned()).collect();
                if parameters.contains_key("desc") {
                    sorted.reverse()
                }
            }
        } else if let (Some(item), expired) = db.check_timeout_item(key) {
            if expired {
                database.write().unwrap().remove_expired_key(key)
            } else if parameters.contains_key("desc") {
                sorted = item.sort_descending();
            } else {
                sorted = item.sort();
            }
        }
        if (parameters.contains_key("lower")) && (parameters.contains_key("upper")) {
            if let RespType::RBulkString(lower_bound) = parameters.get("lower").unwrap() {
                if let RespType::RBulkString(upper_bound) = parameters.get("upper").unwrap() {
                    let min = lower_bound.parse::<usize>().unwrap_or(0);
                    let max = upper_bound.parse::<usize>().unwrap_or(sorted.len());
                    sorted = sorted[min..max].to_vec();
                }
            }
        }
        RespType::RArray(
            sorted
                .iter()
                .map(|e| RespType::RBulkString(e.to_string()))
                .collect(),
        )
    } else {
        RespType::RError("Invalid request".to_string())
    }
}

/// Genera un hashmap a partir de los parámetros ingresados por el usuario.
///
/// Los parámetros pueden ser:
///
/// * ASC | DESC: Ordena de menor a mayor (asc) o de mayor a menor (desc).
/// * ALPHA: Ordena alfabeticamente.
/// * LIMIT lower count: Limita la cantidad de elementos. Toma `count` elementos desde la posicion `lower`.
/// * BY pattern: Permite ordenar a partir de claves externas y sus valores asociados.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::services::commands::command_key;
/// # use std::collections::HashMap;
///
/// let cmd = vec![
///     RespType::RBulkString("SORT".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("ASC".to_string()),
///     ];
///
/// let asc_map = command_key::generate_hashmap(&cmd);
/// let mut map = HashMap::new();
/// let key = RespType::RBulkString(String::from("frutas"));
/// map.insert(String::from("key"), &key);
/// map.insert(String::from("asc"), &RespType::RInteger(1));
/// assert_eq!(asc_map, map);
///
/// let cmd = vec![
///     RespType::RBulkString("SORT".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("BY".to_string()),
///     RespType::RBulkString("max*".to_string()),
///     ];
///
/// let by_map = command_key::generate_hashmap(&cmd);
/// let mut map = HashMap::new();
/// let key = RespType::RBulkString(String::from("frutas"));
/// map.insert(String::from("key"), &key);
/// let by = RespType::RBulkString(String::from("max*"));
/// map.insert(String::from("by"), &by);
/// assert_eq!(by_map, map);
///
/// let cmd = vec![
///     RespType::RBulkString("SORT".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("BY".to_string()),
///     RespType::RBulkString("max*".to_string()),
///     RespType::RBulkString("LIMIT".to_string()),
///     RespType::RBulkString("0".to_string()),
///     RespType::RBulkString("10".to_string()),
///     ];
///
/// let multi_map = command_key::generate_hashmap(&cmd);
/// let mut map = HashMap::new();
/// let key = RespType::RBulkString(String::from("frutas"));
/// map.insert(String::from("key"), &key);
/// let by = RespType::RBulkString(String::from("max*"));
/// map.insert(String::from("by"), &by);
/// let lower = RespType::RBulkString(String::from("0"));
/// map.insert(String::from("lower"), &lower);
/// let upper = RespType::RBulkString(String::from("10"));
/// map.insert(String::from("upper"), &upper);
/// assert_eq!(multi_map, map);
/// ```
pub fn generate_hashmap(cmd: &[RespType]) -> HashMap<String, &RespType> {
    let mut aux_hash_map = HashMap::new();
    let mut pos = 1;
    while pos < cmd.len() {
        if let RespType::RBulkString(arg) = &cmd[pos] {
            let arg = arg.to_lowercase();
            if (arg == "asc") || (arg == "desc") || (arg == "alpha") {
                aux_hash_map.insert(arg.to_string(), &RespType::RInteger(1));
                pos += 1;
            } else if (arg == "by") || (arg == "store") {
                aux_hash_map.insert(arg.to_string(), &cmd[pos + 1]);
                pos += 2;
            } else if arg == "limit" {
                aux_hash_map.insert("lower".to_string(), &cmd[pos + 1]);
                aux_hash_map.insert("upper".to_string(), &cmd[pos + 2]);
                pos += 3;
            } else {
                aux_hash_map.insert("key".to_string(), &cmd[pos]);
                pos += 1;
            }
        }
    }
    aux_hash_map
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
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_keys.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
///  database.write().unwrap().add("animal".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("oso"))
///  ).build());
///  database.write().unwrap().add("animacion".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("soul"))
///  ).build());
///  database.write().unwrap().add("comida".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("pizza"))
///  ).build());
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
        let db = database.read().unwrap();
        let matching_keys = db.get_keys_that_match_pattern(pattern);
        let vec = matching_keys
            .iter()
            .map(|k| RespType::RBulkString(k.to_string()))
            .collect();
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
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, ValueTimeItem, KeyAccessTime, ValueTimeItemBuilder};
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
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()])
/// ).build());
///
/// database.write().unwrap().add("verduras".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["acelga".to_string(),"cebolla".to_string(),"zanahoria".to_string()])).with_timeout(timeout_10seg).build()
/// );
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
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, ValueTimeItem, KeyAccessTime, ValueTimeItemBuilder};
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
/// database.write().unwrap().add("verduras".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["acelga".to_string(),"cebolla".to_string(),"zanahoria".to_string()])).with_timeout(timeout_now).build()
/// );
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
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_ttl.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("pera"))).with_timeout(1925487534).build());
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
            let db = database.read().unwrap();
            return if let (Some(item), expire) = db.check_timeout_item(key) {
                if expire {
                    database.write().unwrap().remove_expired_key(key);
                    RespType::RSignedNumber(-2)
                } else {
                    match item.get_timeout() {
                        KeyAccessTime::Volatile(timeout) => RespType::RInteger(*timeout as usize),
                        KeyAccessTime::Persistent => RespType::RSignedNumber(-1),
                    }
                }
            } else {
                RespType::RSignedNumber(-2)
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
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_type.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// database.write().unwrap().add("fruta".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType(String::from("pera"))
/// ).build());
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
        let (exist, expired) = database
            .read()
            .unwrap()
            .key_exists_expired(current_key.to_string());
        if exist {
            if expired {
                database.write().unwrap().remove_expired_key(current_key)
            } else {
                tipo = database
                    .read()
                    .unwrap()
                    .get_type_of_value(current_key.to_string());
            }
        }
    }
    RespType::RBulkString(tipo)
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
