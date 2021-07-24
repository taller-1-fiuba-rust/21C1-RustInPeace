//! Servicio que implementa todos los comandos de tipo String.

use crate::domain::entities::key_value_item::ValueTimeItemBuilder;
use crate::domain::entities::key_value_item::ValueType;
use crate::errors::database_error::DatabaseError;
use crate::{domain::implementations::database::Database, services::utils::resp_type::RespType};
use std::vec;
use std::{
    convert::TryInto,
    sync::{Arc, RwLock},
};

/// Concatena el valor especificado al final del string almacenado en `key`.
///
/// Si la clave no existe, se crea con un string vacío como valor, luego se le concatena el valor especificado.
/// Retorna el largo del string luego de realizar la concatenación.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::commands::command_string;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType, KeyAccessTime};
///
/// # let db = Database::new("dummy_db_append.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("animal".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType("puerco".to_string())
/// ).build());
///
/// let res = command_string::append(&vec![
///     RespType::RBulkString("APPEND".to_string()),
///     RespType::RBulkString("animal".to_string()),
///     RespType::RBulkString("espin".to_string()),
/// ], &database);
///
/// # match res {
/// #    RespType::RInteger(len) => {
///     assert_eq!(len, 11)
/// # }
/// #    _ => assert!(false)
/// # }
///
/// let _ = std::fs::remove_file("dummy_db_append.csv");
/// ```
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

/// Decrementa el valor almacenado en `key` en `decr` unidades.
///
/// Si la clave no existe, se crea con valor 0 antes de realizar la operación.
/// Retorna el resultado de la operación.
/// Devuelve error si el valor almacenado en `key` no es de tipo string o si no se puede representar como número entero.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::commands::command_string;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType, KeyAccessTime};
///
/// # let db = Database::new("dummy_db_decrby.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("edad".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType("30".to_string())
/// ).build());
///
/// let res = command_string::decrby(&vec![
///     RespType::RBulkString("DECRBY".to_string()),
///     RespType::RBulkString("edad".to_string()),
///     RespType::RBulkString("7".to_string()),
/// ], &database);
///
/// # match res {
/// #    RespType::RSignedNumber(new_edad) => {
///     assert_eq!(new_edad, 23)
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # std::fs::remove_file("dummy_db_decrby.csv");
/// ```
pub fn decrby(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 2 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            if let RespType::RBulkString(decr) = &cmd[2] {
                let number = decr.parse::<i64>();
                return match number {
                    Ok(decr) => match db.decrement_key_by(key, decr) {
                        Ok(res) => RespType::RSignedNumber(res.try_into().unwrap()),
                        Err(e) => RespType::RError(e.to_string()),
                    },
                    Err(e) => RespType::RError(e.to_string()),
                };
            }
        }
    }
    RespType::RError(String::from("Invalid command decrby"))
}

/// Incrementa el valor almacenado en `key` en `incr` unidades.
///
/// Si la clave no existe, se crea con valor 0 antes de realizar la operación.
/// Retorna el resultado de la operación.
/// Devuelve error si el valor almacenado en `key` no es de tipo string o si no se puede representar como número entero.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::commands::command_string;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType, KeyAccessTime};
///
/// # let db = Database::new("dummy_db_incrby.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("edad".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType("30".to_string())
/// ).build());
///
/// let res = command_string::incrby(&vec![
///     RespType::RBulkString("INCRBY".to_string()),
///     RespType::RBulkString("edad".to_string()),
///     RespType::RBulkString("7".to_string()),
/// ], &database);
///
/// # match res {
/// #    RespType::RInteger(new_edad) => {
///     assert_eq!(new_edad, 37)
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # std::fs::remove_file("dummy_db_incrby.csv");
/// ```
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

/// Devuelve el valor almacenado en `key`.
///
/// Si la clave no existe, devuelve `nil`.
/// Devuelve error si el valor almacenado en `key` no es de tipo string.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::commands::command_string;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType, KeyAccessTime};
///
/// # let db = Database::new("dummy_db_get.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("nombre".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType("alfonso".to_string())
/// ).build());
///
/// let res = command_string::get(&vec![
///     RespType::RBulkString("GET".to_string()),
///     RespType::RBulkString("nombre".to_string()),
/// ], &database);
///
/// # match res {
/// #    RespType::RBulkString(n) => {
///     assert_eq!(n, "alfonso".to_string())
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # std::fs::remove_file("dummy_db_get.csv");
/// ```
pub fn get(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            return match db.get_string_value_by_key(key) {
                Some(str) => RespType::RBulkString(str),
                None => RespType::RNullBulkString(),
            };
        }
    }
    RespType::RError(String::from("Invalid command get"))
}

/// Devuelve los valores almacenados en las claves especificadas.
///
/// Si la clave no existe o el valor que almacena no es de tipo string, devuelve `nil`.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::commands::command_string;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType, KeyAccessTime};
///
/// # let db = Database::new("dummy_db_mget.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("nombre".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType("alfonso".to_string())
/// ).build());
/// database.write().unwrap().add("apellido".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType("alvarez".to_string())
/// ).build());
///
/// let res = command_string::mget(&vec![
///     RespType::RBulkString("MGET".to_string()),
///     RespType::RBulkString("nombre".to_string()),
///     RespType::RBulkString("apellido".to_string()),
/// ], &database);
///
/// # match res {
/// #    RespType::RArray(full_name) => {
///    assert_eq!(full_name, vec![RespType::RBulkString("alfonso".to_string()), RespType::RBulkString("alvarez".to_string())])
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # std::fs::remove_file("dummy_db_mget.csv");
/// ```
pub fn mget(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut db = database.write().unwrap();
    let mut vec_keys_with_string_values = vec![];
    if cmd.len() > 1 {
        for n in cmd.iter().skip(1) {
            if let RespType::RBulkString(current_key) = n {
                if db.key_exists(current_key.to_string()) {
                    if let Some(actual_value) = db.get_string_value_by_key(current_key) {
                        vec_keys_with_string_values.push(RespType::RBulkString(actual_value));
                    }
                }
            }
        }
        RespType::RArray(vec_keys_with_string_values)
    } else {
        RespType::RError(String::from("Invalid command get"))
    }
}

/// Devuelve el valor almacenado en `key` y lo elimina.
///
/// Si la clave no existe, devuelve `nil`.
/// Devuelve error si el valor almacenado en `key` no es de tipo string.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::commands::command_string;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType, KeyAccessTime};
///
/// # let db = Database::new("dummy_db_getdel.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("nombre".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType("alfonso".to_string())
/// ).build());
///
/// let res = command_string::getdel(&vec![
///     RespType::RBulkString("GETDEL".to_string()),
///     RespType::RBulkString("nombre".to_string()),
///     RespType::RBulkString("alfredo".to_string()),
/// ], &database);
///
/// # match res {
/// #    RespType::RBulkString(old_name) => {
///         assert_eq!(old_name, "alfonso".to_string())
/// #    }
/// #    _ => assert!(false)
/// # }
///
/// # std::fs::remove_file("dummy_db_getdel.csv");
/// ```
pub fn getdel(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            match db.getdel_value_by_key(key) {
                Ok(str) => {
                    return RespType::RBulkString(str);
                }
                Err(e) => {
                    if let DatabaseError::MissingKey() = e {
                        return RespType::RNullBulkString();
                    }
                    return RespType::RError(e.to_string());
                }
            }
        }
    }
    RespType::RError(String::from("Invalid command getdel"))
}

/// Actualiza el valor almacenado en `key` y devuelve el valor anterior.
///
/// Si la clave no existe, devuelve `nil`.
/// Devuelve error si el valor almacenado en `key` no es de tipo string.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::commands::command_string;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType, KeyAccessTime};
///
/// # let db = Database::new("dummy_db_getset.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("nombre".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType("alfonso".to_string())
/// ).build());
///
/// let res = command_string::getdel(&vec![
///     RespType::RBulkString("GETSET".to_string()),
///     RespType::RBulkString("nombre".to_string()),
///     RespType::RBulkString("alfredo".to_string()),
/// ], &database);
///
/// # match res {
/// #    RespType::RBulkString(old_name) => {
///         assert_eq!(old_name, "alfonso".to_string())
/// #    }
/// #    _ => assert!(false)
/// # }
///
/// # std::fs::remove_file("dummy_db_getset.csv");
/// ```
pub fn getset(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            if let RespType::RBulkString(new_value) = &cmd[2] {
                match db.getset_value_by_key(key, new_value) {
                    Ok(str) => {
                        return RespType::RBulkString(str);
                    }
                    Err(e) => {
                        if let DatabaseError::MissingKey() = e {
                            return RespType::RNullBulkString();
                        }
                        return RespType::RError(e.to_string());
                    }
                }
            }
        }
    }
    RespType::RError(String::from("Invalid command getset"))
}

/// Devuelve la longitud del valor almacenado en `key`.
///
/// Si la clave no existe, devuelve 0.
/// Devuelve error si el valor almacenado no es de tipo string.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::commands::command_string;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType, KeyAccessTime};
///
/// # let db = Database::new("dummy_db_strlen.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("animal".to_string(),ValueTimeItemBuilder::new(
///     ValueType::StringType("puerco".to_string())
/// ).build());
///
/// let res = command_string::strlen(&vec![
///     RespType::RBulkString("STRLEN".to_string()),
///     RespType::RBulkString("animal".to_string()),
/// ], &database);
///
/// # match res {
/// #    RespType::RInteger(len) => {
///        assert_eq!(len, 6)
/// #    }
/// #    _ => assert!(false)
/// # }
///
/// # let _ = std::fs::remove_file("dummy_db_strlen.csv");
/// ```
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

/// Actualiza el valor de las claves especificadas.
///
/// Equivale a aplicar la función `set` a múltiples valores.
/// No es posible saber si alguna clave no se actualizó.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::commands::command_string;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
///
/// # let db = Database::new("dummy_db_mset.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// let res = command_string::mset(&vec![
///     RespType::RBulkString("MSET".to_string()),
///     RespType::RBulkString("nombre".to_string()),
///     RespType::RBulkString("alfredo".to_string()),
///     RespType::RBulkString("apellido".to_string()),
///     RespType::RBulkString("alvarez".to_string()),
/// ], &database);
///
/// # match res {
/// #    RespType::RBulkString(response) => {
///         assert_eq!(response, "Ok".to_string())
/// #    }
/// #    _ => assert!(false)
/// # }
///
/// # std::fs::remove_file("dummy_db_mset.csv");
/// ```
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
            let vt_item =
                ValueTimeItemBuilder::new(ValueType::StringType(vec_aux[pos + 1].to_string()))
                    .build();
            db.add(e.to_string(), vt_item);
        }
        RespType::RBulkString("Ok".to_string())
    } else {
        RespType::RBulkString("One or more parameters are missing".to_string())
    }
}

/// Actualiza el valor de las clave especificada.
///
/// Si la clave ya contenía un valor, lo reemplaza sin importar el tipo de dato.
/// Admite los siguientes parámetros:
/// * EX: Tiempo de expiración en segundos.
/// * PX: Tiempo de expiración en milisegundos.
/// * EXAT: Tiempo UNIX en que va a expirar la clave, en segundos.
/// * PXAT: Tiempo UNIX en que va a expirar la clave, en milisegundos.
/// * NX: Actualiza la clave solo si no existia previamente.
/// * XX: Actualiza la clave solo si ya existía previamente.
///
/// Esta función devuelve `Ok` si la clave fue actualizada correctamente.
/// Si recibe el parametro NX o XX y no se cumple la condición, devuelve `nil`.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::commands::command_string;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
///
/// # let db = Database::new("dummy_db_set.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
///
/// let res = command_string::set(&vec![
///     RespType::RBulkString("SET".to_string()),
///     RespType::RBulkString("nombre".to_string()),
///     RespType::RBulkString("alfredo".to_string()),
/// ], &database);
///
/// # match res {
/// #    RespType::RBulkString(response) => {
///         assert_eq!(response, "Ok".to_string())
/// #    }
/// #    _ => assert!(false)
/// # }
///
/// # std::fs::remove_file("dummy_db_set.csv");
/// ```
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

/// Devuelve un vector con los parámetros especificados por el usuario.
///
/// Los parámetros se dividen en tres grupos:
/// * EX | PX | EXAT | PXAT
/// * NX | XX
/// * GET
///
/// Esta función devuelve un vector de tres elementos, cada elemento representa un parámetro y su valor.
/// Por defecto, el valor de un parámetro es None.
///
/// # Ejemplo
/// ```
/// # use proyecto_taller_1::services::commands::command_string;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
///
/// let cmd = vec![
///     RespType::RBulkString("SET".to_string()),
///     RespType::RBulkString("nombre".to_string()),
///     RespType::RBulkString("alfredo".to_string()),
///     RespType::RBulkString("px".to_string()),
///     RespType::RBulkString("10".to_string()),
///     RespType::RBulkString("xx".to_string()),
///     RespType::RBulkString("get".to_string())];
/// let res = command_string::generate_options(&cmd);
/// assert_eq!(res, vec![(String::from("px"), Some(&String::from("10"))), (String::from("set_if_exists"), Some(&String::from("xx"))), (String::from("get_old_value"), Some(&String::from("get")))]);
/// ```
pub fn generate_options(cmd: &[RespType]) -> Vec<(String, Option<&String>)> {
    let mut options = vec![
        (String::from("expire_at"), None),
        (String::from("set_if_exists"), None),
        (String::from("get_old_value"), None),
    ];
    for (pos, argumento) in cmd.iter().skip(3).enumerate() {
        println!("pos: {}", pos);
        if let RespType::RBulkString(arg) = argumento {
            if (arg == "ex") || (arg == "px") || (arg == "exat") || (arg == "pxat") {
                if let RespType::RBulkString(expire_at) = &cmd[pos + 4] {
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
