//! Servicio que implementa todos los comandos de tipo List.

use crate::domain::entities::key_value_item::ValueType;
use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
use std::sync::{Arc, RwLock};
use std::usize;

/// Retorna la longitud de la lista almacenada en la clave especificada.
///
/// Si la clave no existe, retorna 0.
/// Si el valor almacenado no es de tipo Lista, retorna Error.
/// # Ejemplo:
/// ```
/// # use proyecto_taller_1::services::commands::command_list;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_llen.csv".to_string());
/// let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()])
/// ).build());
///
/// let res = command_list::llen(&vec![
///     RespType::RBulkString("LLEN".to_string()),
///     RespType::RBulkString("frutas".to_string()),
/// ], &database);
///
/// # match res {
/// # RespType::RInteger(len) => {
/// assert_eq!(len, 3)
///# }
/// # _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_llen.csv");
/// ```
pub fn llen(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let new_database = database
        .read()
        .expect("Could not get database lock on llen");
    if let RespType::RBulkString(key) = &cmd[1] {
        if let (Some(item), false) = new_database.check_timeout_item(key) {
            if let ValueType::ListType(current_value) = item.get_value().to_owned() {
                let list_size = current_value.len();
                RespType::RInteger(list_size)
            } else {
                RespType::RError("Not list type".to_string())
            }
        } else {
            if let (Some(_), true) = new_database.check_timeout_item(key) {
                drop(new_database);
                database.write().unwrap().remove_expired_key(key)
            }
            RespType::RInteger(0)
        }
    } else {
        RespType::RError("Invalid request".to_string())
    }
}

/// Elimina y devuelve los primeros elementos de la lista almacenada en `key`.
///
/// Por defecto, elimina el primer elemento de la lista. Si se le pasa el parámetro opcional `count`, elimina
/// los primeros `count` elementos.
/// Si `count` no puede representarse como un numero entero sin signo, se considera 1 por defecto.
/// Si la clave no existe, retorna `nil`. Si existe y `count` es mayor a 1 retorna un array con los elementos eliminados.
/// Si existe y no recibe el parámetro `count`, devuelve un bulkstring con el valor del primer elemento.
/// Ante un error inesperado, devuelve Error `Invalid request`.
///
/// # Ejemplos
///
/// ```
/// # use proyecto_taller_1::services::commands::command_list;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_lpop.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string(), "melon".to_string(), "ciruela".to_string()])
/// ).build());
///
/// let res = command_list::lpop(&vec![
///     RespType::RBulkString("LPOP".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("3".to_string())
/// ], &database);
///
/// # match res {
/// #    RespType::RArray(frutas_eliminadas) => {
///     assert_eq!(frutas_eliminadas, vec![RespType::RBulkString("kiwi".to_string()),RespType::RBulkString("pomelo".to_string()), RespType::RBulkString("sandia".to_string())])
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # let _ = std::fs::remove_file("dummy_db_lpop.csv");
/// ```
pub fn lpop(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut db = database
        .write()
        .expect("Could not get database lock on lpop");
    if let RespType::RBulkString(key) = &cmd[1] {
        if cmd.len() == 3 {
            if let RespType::RBulkString(cantidad) = &cmd[2] {
                let popped_elements =
                    db.pop_elements_from_list(key, cantidad.parse::<usize>().unwrap_or(1));
                if let Some(popped) = popped_elements {
                    return RespType::RArray(
                        popped
                            .iter()
                            .map(|element| RespType::RBulkString(element.to_string()))
                            .collect(),
                    );
                } else {
                    return RespType::RNullBulkString();
                }
            }
        } else {
            let popped_elements = db.pop_elements_from_list(key, 1);
            if popped_elements.is_none() {
                return RespType::RBulkString("()".to_string());
            }
            if let Some(popped_element) = popped_elements {
                if !popped_element.is_empty() {
                    return RespType::RBulkString(popped_element[0].to_owned());
                }
            }
        }
    }
    RespType::RError("Invalid request".to_string())
}

/// Guarda los elementos enviados por parámetro en una lista.
///
/// A partir de una `key` dada, se agregan los elementos que se envían. En caso que la lista no exista, se crea.
/// Si el valor almacenado no es una lista, se devuelve un error.
/// Hay dos formas de guardar los datos enviados: si se envía is_reverse en true, se guardarán de izquierda
/// a derecha desde el head de la lista. En caso que is_reverse venga en false se insertarán desde el fondo de la lista.
/// Devuelve la longitud de la lista luego de haber insertado los nuevos elementos.
///
/// # Ejemplo:
/// ```
/// # use proyecto_taller_1::services::commands::command_list;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_push.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()])
/// ).build());
///
/// let res = command_list::push(&vec![
///     RespType::RBulkString("LPUSH".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("frutilla".to_string()),
///     RespType::RBulkString("melon".to_string()),
/// ], &database, true);
///
/// # match res {
/// #    RespType::RInteger(len) => {
/// assert_eq!(len, 5)
///# }
/// # _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_push.csv");
/// ```
pub fn push(cmd: &[RespType], database: &Arc<RwLock<Database>>, is_reverse: bool) -> RespType {
    let mut new_database = database
        .write()
        .expect("Could not get database lock on push");
    let mut vec_aux = vec![];
    if let RespType::RBulkString(key) = &cmd[1] {
        if is_reverse {
            for n in cmd.iter().skip(2).rev() {
                if let RespType::RBulkString(value) = n {
                    vec_aux.push(value.to_string());
                }
            }
        } else {
            for n in cmd.iter().skip(2) {
                if let RespType::RBulkString(value) = n {
                    vec_aux.push(value.to_string());
                }
            }
        }

        if let Some(resultado) = new_database.add_to_list_type(vec_aux, key, false) {
            RespType::RInteger(resultado)
        } else {
            RespType::RError("error - not list type".to_string())
        }
    } else {
        RespType::RError("Invalid request".to_string())
    }
}

/// Inserta los valores especificados al comienzo de la lista almacenada en `key`.
///
/// Si la clave existe y guarda un elemento de tipo lista, inserta los elementos al comienzo de la misma.
/// Retorna un valor de tipo entero que representa la longitud de la lista luego de haber insertado los nuevos elementos.
/// Si la clave no es de tipo lista, devuelve 0.
/// Ante un error inesperado, devuelve Error `Invalid request`.
///
/// # Ejemplos
///
/// ```
/// # use proyecto_taller_1::services::commands::command_list;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_lpushx.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()]),
/// ).build());
///
/// let res = command_list::lpushx(&vec![
///     RespType::RBulkString("LPUSHX".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("melon".to_string()),
///     RespType::RBulkString("sandia".to_string())
/// ], &database);
///
/// # match res {
/// #    RespType::RInteger(len) => {
///     assert_eq!(len, 5)
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # let _ = std::fs::remove_file("dummy_db_lpushx.csv");
/// ```
pub fn lpushx(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut new_database = database
        .write()
        .expect("Could not get database lock on lpushx");
    let mut vec_aux = vec![];
    if let RespType::RBulkString(key) = &cmd[1] {
        for n in cmd.iter().skip(2).rev() {
            if let RespType::RBulkString(value) = n {
                vec_aux.push(value.to_string());
            }
        }
        let resultado = new_database
            .add_to_list_type(vec_aux, key, true)
            .unwrap_or(0);
        RespType::RInteger(resultado)
    } else {
        RespType::RError("Invalid request".to_string())
    }
}

/// Devuelve elementos de la lista almacenada en `key` dentro del rango especificado.
///
/// Busca los elementos posicionados entre `start` y `stop`, tal que 0 es la posición del primer elemento,
/// 1 la posición del segundo elemento, etc. Una posición negativa indica que se deben contar las posiciones
/// desde el final de la lista, por ejemplo -1 es la posición del último elemento, -2 la posición del penúltimo, etc.
/// Si `start` es mayor a la longitud de la lista, la función retorna una lista vacía.
/// Si `stop` es mayor a la longitud total de la lista, se lo va a considerar como el último elemento.
/// Retorna todos los elementos pertenecientes al rango entre `start` y `stop`.
///
/// # Ejemplo
///
/// ```
/// # use proyecto_taller_1::services::commands::command_list;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_lrange.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string(), "melon".to_string(), "ciruela".to_string()])
/// ).build());
///
/// let res = command_list::lrange(&vec![
///     RespType::RBulkString("LRANGE".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("1".to_string()),
///     RespType::RBulkString("3".to_string())
/// ], &database);
///
/// # match res {
/// #    RespType::RArray(frutas_range) => {
///     assert_eq!(frutas_range, vec![RespType::RBulkString("pomelo".to_string()),RespType::RBulkString("sandia".to_string()), RespType::RBulkString("melon".to_string())])
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # let _ = std::fs::remove_file("dummy_db_lrange.csv");
/// ```
pub fn lrange(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let new_database = database
        .read()
        .expect("Could not get database lock on lrange");
    if let RespType::RBulkString(key) = &cmd[1] {
        if let RespType::RBulkString(lower_bound) = &cmd[2] {
            if let RespType::RBulkString(upper_bound) = &cmd[3] {
                if let Some(value_vec) =
                    new_database.get_values_in_range(key, lower_bound, upper_bound)
                {
                    let mut value_vec_resptype = vec![];
                    for elemento in value_vec {
                        value_vec_resptype.push(RespType::RBulkString(elemento));
                    }
                    RespType::RArray(value_vec_resptype)
                } else {
                    RespType::RBulkString("error".to_string())
                }
            } else {
                RespType::RBulkString("No upper_bound_specified".to_string())
            }
        } else {
            RespType::RBulkString("No lower_bound_specified".to_string())
        }
    } else {
        RespType::RError("Invalid request".to_string())
    }
}

/// Devuelve el valor en la posición `index` de la lista asociada a una `key`.
///
/// A partir de una `key` dada, se busca la lista asociada y se devuelve
/// el valor ubicado en la posición `index`.
/// Si la key no existe se devuelve un nill
/// Si la cantidad de parámetros enviados en `cmd` son
/// incorrectos se retorna un error con mensaje informando el problema.
/// Si el valor asociado a la `key` no es una lista también se devuelve
/// un error.
/// Las respuestas válidas son: el elemento encontrado en la posición
/// `index` o `nil` si el `index` no pertenece al rango de la lista.
/// Además  `index` en caso de ser un número negativo hace referencia a
/// la posición empezando desde la cola.
///
/// # Ejemplos
///
/// Primer item de la lista con key `frutas`:
///
/// ```
/// # use proyecto_taller_1::services::commands::command_list;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_doc1.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()])
/// ).build());
///
/// // key: "frutas , index: "0"
/// let res = command_list::lindex(&vec![
///     RespType::RBulkString("LINDEX".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("0".to_string())
/// ], &database);
///
/// # match res {
/// #    RespType::RBulkString(fruta) => {
///     assert_eq!(fruta, "kiwi")
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # let _ = std::fs::remove_file("dummy_db_doc1.csv");
/// ```
/// Último elemento de la lista con key `frutas`:
///
/// ```
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem, ValueTimeItemBuilder};
/// # use proyecto_taller_1::services::commands::command_list;
///
/// # let db = Database::new("dummy_db_doc2.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()])
/// ).build());
///
/// // key: "frutas , index: "-1"
/// let res = command_list::lindex(&vec![
///     RespType::RBulkString("LINDEX".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("-1".to_string())
/// ], &database);
///
/// # match res {
/// #    RespType::RBulkString(fruta) => {
/// assert_eq!(fruta, "sandia")
/// # },
/// #    _ => assert!(false)
/// # }
/// # let _ = std::fs::remove_file("dummy_db_doc2.csv");
/// ```
pub fn lindex(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() == 3 {
        let db = database
            .read()
            .expect("Could not get database read lock on lindex");
        if let RespType::RBulkString(key) = &cmd[1] {
            if let RespType::RBulkString(index) = &cmd[2] {
                let (item, expire) = db.check_timeout_item(key);
                if item.is_some() && expire {
                    drop(db);
                    database
                        .write()
                        .expect("Could not get database write lock on lindex")
                        .remove_expired_key(key)
                }
                let db = database
                    .read()
                    .expect("Could not get database read lock on lindex");
                let current_value_in_list_by_index = db.get_value_by_index(key, index);
                return if let Some(value) = current_value_in_list_by_index {
                    RespType::RBulkString(value)
                } else {
                    RespType::RError("value is not list type".to_string())
                };
            }
        }
    }
    RespType::RError("Invalid request".to_string())
}

/// Elimina las primeras `count` ocurrencias del elemento especificado perteneciente a la lista almacenada en `key`.
///
/// Si `count` es mayor a 0, elimina aquellos elementos leyendo la lista de izquierda a derecha.
/// Si `count` es menor a 0, elimina aquellos elementos leyendo la lista de derecha a izquierda.
/// Si `count` es igual a 0, elimina todos los elementos que coincidan con el especificado.
/// Si `count` no puede representarse como un número entero, se asigna 1.
/// Si `key` no existe, retorna 0.
/// Retorna la cantidad de elementos eliminados de la lista.
///
/// # Ejemplo
///
/// ```
/// # use proyecto_taller_1::services::commands::command_list;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_lrem.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string(),"pomelo".to_string(),"sandia".to_string()])
/// ).build());
///
/// let res = command_list::lrem(&vec![
///     RespType::RBulkString("LREM".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("2".to_string()),
///     RespType::RBulkString("pomelo".to_string())
/// ], &database);
///
/// # match res {
/// #    RespType::RInteger(len) => {
///     assert_eq!(len, 2)
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # let _ = std::fs::remove_file("dummy_db_lrem.csv");
/// ```
pub fn lrem(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() == 4 {
        let mut db = database
            .write()
            .expect("Could not get database lock on lrem");
        if let RespType::RBulkString(key) = &cmd[1] {
            if let RespType::RBulkString(count) = &cmd[2] {
                if let RespType::RBulkString(element) = &cmd[3] {
                    RespType::RInteger(db.delete_elements_of_value_list(
                        key,
                        count.to_string(),
                        element.to_string(),
                    ))
                } else {
                    RespType::RInteger(0)
                }
            } else {
                RespType::RInteger(0)
            }
        } else {
            RespType::RInteger(0)
        }
    } else {
        RespType::RError("Invalid request".to_string())
    }
}

/// Actualiza el elemento que se encuentra en la posición `index` de la lista.
///
/// Devuelve error si `index` no pertenece al rango de la lista.
/// Retorna "Ok" en caso de exito.
///
/// # Ejemplo
///
/// ```
/// # use proyecto_taller_1::services::commands::command_list;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType};
///
/// # let db = Database::new("dummy_db_lset.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()])
/// ).build());
///
/// let res = command_list::lset(&vec![
///     RespType::RBulkString("LSET".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("0".to_string()),
///     RespType::RBulkString("manzana".to_string())
/// ], &database);
///
/// # match res {
/// #    RespType::RBulkString(response) => {
///     assert_eq!(response, "Ok".to_string())
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # let _ = std::fs::remove_file("dummy_db_lset.csv");
/// ```
pub fn lset(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() == 4 {
        let mut db = database
            .write()
            .expect("Could not get database lock on lset");
        if let RespType::RBulkString(key) = &cmd[1] {
            if let RespType::RBulkString(index) = &cmd[2] {
                if let RespType::RBulkString(value) = &cmd[3] {
                    let succeful_replace = db.replace_element_in_list_type_value(key, value, index);
                    if succeful_replace {
                        return RespType::RBulkString("Ok".to_string());
                    } else {
                        return RespType::RError("out of bounds".to_string());
                    }
                }
            }
        }
    }
    RespType::RBulkString("Invalid request".to_string())
}

/// Elimina y devuelve los últimos elementos de la lista almacenada en `key`.
///
/// Por defecto, elimina el último elemento de la lista. Si se le pasa el parámetro opcional `count`, elimina
/// los últimos `count` elementos.
/// Si `count` no puede representarse como un numero entero sin signo, se considera 1 por defecto.
/// Si la clave no existe, retorna `nil`. Si existe y `count` es mayor a 1 retorna un array con los elementos eliminados.
/// Si existe y no recibe el parámetro `count`, devuelve un bulkstring con el valor del último elemento.
/// Ante un error inesperado, devuelve Error `Invalid request`.
///
/// # Ejemplo
///
/// ```
/// # use proyecto_taller_1::services::commands::command_list;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_rpop_command.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string(), "melon".to_string(), "ciruela".to_string()])
/// ).build());
///
/// let res = command_list::rpop(&vec![
///     RespType::RBulkString("RPOP".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("3".to_string())
/// ], &database);
///
/// # match res {
/// #    RespType::RArray(frutas_eliminadas) => {
///     assert_eq!(frutas_eliminadas, vec![RespType::RBulkString("ciruela".to_string()),RespType::RBulkString("melon".to_string()), RespType::RBulkString("sandia".to_string())])
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # let _ = std::fs::remove_file("dummy_db_rpop_command.csv");
/// ```
pub fn rpop(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut db = database
        .write()
        .expect("Could not get database lock on rpop");
    if let RespType::RBulkString(key) = &cmd[1] {
        if cmd.len() == 3 {
            if let RespType::RBulkString(cantidad) = &cmd[2] {
                let popped_elements =
                    db.rpop_elements_from_list(key, cantidad.parse::<usize>().unwrap_or(1));
                if let Some(popped) = popped_elements {
                    return RespType::RArray(
                        popped
                            .iter()
                            .map(|element| RespType::RBulkString(element.to_string()))
                            .collect(),
                    );
                } else {
                    return RespType::RNullBulkString();
                }
            }
        } else {
            let popped_elements = db.rpop_elements_from_list(key, 1);
            if let Some(popped_element) = popped_elements {
                if !popped_element.is_empty() {
                    return RespType::RBulkString(popped_element[0].to_owned());
                }
            } else {
                return RespType::RNullBulkString();
            }
        }
    }
    RespType::RError("Invalid request".to_string())
}

/// Inserta los valores especificados al final de la lista almacenada en `key`.
///
/// Si la clave existe y guarda un elemento de tipo lista, inserta los elementos al final de la misma.
/// Retorna un valor de tipo entero que representa la longitud de la lista luego de haber insertado los nuevos elementos.
/// Ante un error inesperado, devuelve Error `Invalid request`.
///
/// # Ejemplo
///
/// ```
/// # use proyecto_taller_1::services::commands::command_list;
/// # use proyecto_taller_1::services::utils::resp_type::RespType;
/// # use proyecto_taller_1::domain::implementations::database::Database;
/// # use std::sync::{Arc, RwLock};
/// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
///
/// # let db = Database::new("dummy_db_rpushx_command.csv".to_string());
/// # let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItemBuilder::new(
///     ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()])
/// ).build());
///
/// let res = command_list::rpushx(&vec![
///     RespType::RBulkString("RPUSHX".to_string()),
///     RespType::RBulkString("frutas".to_string()),
///     RespType::RBulkString("melon".to_string()),
///     RespType::RBulkString("sandia".to_string())
/// ], &database);
///
/// # match res {
/// #    RespType::RInteger(len) => {
///     assert_eq!(len, 5)
/// # }
/// #    _ => assert!(false)
/// # }
///
/// # let _ = std::fs::remove_file("dummy_db_rpushx_command.csv");
/// ```
pub fn rpushx(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut new_database = database
        .write()
        .expect("Could not get database lock on rpushx");
    let mut new_elements = vec![];
    if let RespType::RBulkString(key) = &cmd[1] {
        for n in cmd.iter().skip(2) {
            if let RespType::RBulkString(value) = n {
                new_elements.push(value.to_string());
            }
        }
        RespType::RInteger(new_database.push_vec_to_list(new_elements, key))
    } else {
        RespType::RError("Invalid request".to_string())
    }
}
