use crate::domain::entities::key_value_item::ValueType;
use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::usize;

pub fn llen(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut new_database = database.write().unwrap();
    if let RespType::RBulkString(key) = &cmd[1] {
        if new_database.key_exists(key.to_string()) {
            if let ValueType::ListType(current_value) = new_database
                .get_live_item(key)
                .unwrap()
                .get_value()
                .to_owned()
            {
                let list_size = current_value.len();
                RespType::RBulkString(list_size.to_string())
            } else {
                RespType::RBulkString("error - not list type".to_string())
            }
        } else {
            RespType::RBulkString("0".to_string())
        }
    } else {
        RespType::RError("empty request".to_string())
    }
}

pub fn lpop(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut db = database.write().unwrap();
    if let RespType::RBulkString(key) = &cmd[1] {
        if cmd.len() == 3 {
            if let RespType::RBulkString(cantidad) = &cmd[2] {
                let popped_elements =
                    db.pop_elements_from_list(key, cantidad.parse::<usize>().unwrap());
                if let Some(popped) = popped_elements {
                    let mut p = Vec::new();
                    popped.iter().for_each(|element| {
                        p.push(RespType::RBulkString(element.to_string()));
                    });
                    return RespType::RArray(p);
                } else {
                    return RespType::RNullBulkString();
                }
            }
        } else {
            let popped_elements = db.pop_elements_from_list(key, 1);
            if let Some(popped_element) = popped_elements {
                if !popped_element.is_empty() {
                    return RespType::RBulkString(popped_element[0].to_owned());
                }
            }
        }
    }
    RespType::RBulkString("empty".to_string()) //Error o nil?
}

/// Guarda los elementos enviados por parámetro en una lista.
///
/// A partir de una `key` dada, se agregan los elementos que se envían. En caso que la lista no exista, se crea.
/// Si el valor almacenado no es una lista, se devuelve un error.
/// Hay dos formas de guardar los datos enviados: si se envía is_reverse en true, se guardarán de izquierda
/// a derecha desde el head de la lista. En caso que is_reverse venga en false se insertarán desde el fondo de la lista.
///
/// # Example:
///
/// let db = Database::new("dummy_db_doc_list_push.csv".to_string());
/// let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItem::new_now(
/// ValueType::ListType(vec!["kiwi","pomelo","sandia"]),
/// KeyAccessTime::Persistent
/// ));
///
/// let res = command_list::push(&vec![
/// RespType::RBulkString("LPUSH".to_string()),
/// RespType::RBulkString("frutas".to_string()),
/// RespType::RBulkString("frutilla".to_string()),
/// RespType::RBulkString("melon".to_string()),
/// ], &database,true);
///
/// match res {
/// RespType::RInteger(qty) => {
/// assert_eq!(qty,2)
///}
/// _ => assert!(false)
/// }
/// let _ = std::fs::remove_file("dummy_db_doc_list_push.csv");
/// ```

pub fn push(cmd: &[RespType], database: &Arc<RwLock<Database>>, is_reverse: bool) -> RespType {
    let mut new_database = database.write().unwrap();
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

        if let Some(resultado) =
            new_database.push_new_values_into_existing_or_non_existing_key_value_pair(vec_aux, key)
        {
            RespType::RInteger(resultado)
        } else {
            RespType::RBulkString("error - not list type".to_string())
        }
    } else {
        RespType::RError("empty request".to_string())
    }
}

pub fn lpushx(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut new_database = database.write().unwrap();
    let mut vec_aux = vec![];
    if let RespType::RBulkString(key) = &cmd[1] {
        for n in cmd.iter().skip(2).rev() {
            if let RespType::RBulkString(value) = n {
                vec_aux.push(value.to_string());
            }
        }
        if let Some(resultado) =
            new_database.push_new_values_into_existing_key_value_pair(vec_aux, key)
        {
            RespType::RInteger(resultado)
        } else {
            RespType::RBulkString("".to_string())
        }
    } else {
        RespType::RError("empty request".to_string())
    }
}

pub fn lrange(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut new_database = database.write().unwrap();
    //let mut vec_aux = vec![];
    if let RespType::RBulkString(key) = &cmd[1] {
        if let RespType::RBulkString(lower_bound) = &cmd[2] {
            if let RespType::RBulkString(upper_bound) = &cmd[3] {
                if let Some(value_vec) =
                    new_database.get_values_from_list_value_type(key, lower_bound, upper_bound)
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
                RespType::RBulkString("no upper_bound_specified".to_string())
            }
        } else {
            RespType::RBulkString("no lower_bound_specified".to_string())
        }
        // for n in cmd.iter().skip(2).rev() {
        //     if let RespType::RBulkString(value) = n {
        //         vec_aux.push(value.to_string());
        //     }
        // }
        // if let Some(resultado) =
        //     new_database.push_new_values_into_existing_key_value_pair(vec_aux, key)
        // {
        //     RespType::RInteger(resultado)
        // } else {
        //     RespType::RBulkString("".to_string())
        // }
    } else {
        RespType::RError("empty request".to_string())
    }
}

/// Devuelve el valor en la posición `index` de la lista asociada a una `key`.
///
/// A partir de una `key` dada, se busca la lista asociada y se devuelve
/// el valor ubicado en la posición `index`.
/// Si la key no existe o la cantidad de parámetros enviados en `cmd` son
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
/// use proyecto_taller_1::services::commands::command_list;
/// use proyecto_taller_1::services::utils::resp_type::RespType;
/// use proyecto_taller_1::domain::implementations::database::Database;
/// use std::sync::{Arc, RwLock};
/// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
///
/// // Agrego los datos en la base de datos
/// let db = Database::new("dummy_db_doc1.csv".to_string());
/// let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItem::new_now(
/// ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()]),
/// KeyAccessTime::Persistent
/// ));
///
/// //Ejecuto la búsqueda con los parámetros necesarios:
/// // key: "frutas , index: "0"
/// let res = command_list::get_index(&vec![
/// RespType::RBulkString("LINDEX".to_string()),
/// RespType::RBulkString("frutas".to_string()),
/// RespType::RBulkString("0".to_string())
/// ], &database);
///
/// match res {
///     RespType::RBulkString(fruta) => {
///     assert_eq!(fruta, "kiwi") }
///     _ => assert!(false)
/// }
///
/// let _ = std::fs::remove_file("dummy_db_doc1.csv");
/// ```
/// Último elemento de la lista con key `frutas`:
///
/// ```
/// use proyecto_taller_1::services::utils::resp_type::RespType;
/// use proyecto_taller_1::domain::implementations::database::Database;
/// use std::sync::{Arc, RwLock};
/// use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItem};
/// use proyecto_taller_1::services::commands::command_list;
///
/// // Agrego los datos en la base de datos
/// let db = Database::new("dummy_db_doc2.csv".to_string());
/// let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItem::new_now(
/// ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()]),
/// KeyAccessTime::Persistent
/// ));
///
/// // Ejecuto la búsqueda con los parámetros necesarios:
/// // key: "frutas , index: "-1"
/// let res = command_list::get_index(&vec![
/// RespType::RBulkString("LINDEX".to_string()),
/// RespType::RBulkString("frutas".to_string()),
/// RespType::RBulkString("-1".to_string())
/// ], &database);
///
/// match res {
///     RespType::RBulkString(fruta) => { assert_eq!(fruta, "sandia") },
///     _ => assert!(false)
/// }
/// let _ = std::fs::remove_file("dummy_db_doc2.csv");
/// ```
pub fn get_index(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() == 3 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let mut db = database.write().unwrap();
            if let RespType::RBulkString(index) = &cmd[2] {
                let iindex;
                if isize::from_str(index).is_err() {
                    return RespType::RError(isize::from_str(index).err().unwrap().to_string());
                } else {
                    iindex = isize::from_str(index).unwrap();
                }

                return match db.get_live_item(key) {
                    None => RespType::RError(String::from("Key not found")),
                    Some(vti) => {
                        if let ValueType::ListType(items) = vti.get_value() {
                            if iindex.abs() as usize > items.len() {
                                //Fuera de rango
                                return RespType::RNullBulkString();
                            }
                            let i = iindex.abs() as usize;
                            return if iindex >= 0 {
                                //Hago unwrap porque ya chequee el tamaño del vector
                                let string = items.get(i).unwrap();
                                RespType::RBulkString(string.to_string())
                            } else {
                                RespType::RBulkString((&items[items.len() - i]).to_string())
                            };
                        }
                        RespType::RError(String::from("Value is not a list"))
                    }
                };
            }
        }
    }
    RespType::RError(String::from("Invalid command lindex"))
}

pub fn lrem(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() == 4 {
        let mut db = database.write().unwrap();
        if let RespType::RBulkString(key) = &cmd[1] {
            if let RespType::RBulkString(count) = &cmd[2] {
                if let RespType::RBulkString(element) = &cmd[3] {
                    RespType::RInteger(db.delete_elements_of_value_list(
                        key,
                        count.to_string(),
                        element.to_string(),
                    ))
                } else {
                    RespType::RBulkString("0".to_string())
                }
            } else {
                RespType::RBulkString("0".to_string())
            }
        } else {
            RespType::RBulkString("0".to_string())
        }
    } else {
        RespType::RBulkString("incomplete command".to_string())
    }
}

/// Elimina y devuelve los últimos elementos de la lista almacenada en `key`.
///
/// Por defecto, elimina el último elemento de la lista. Si se le pasa el parámetro opcional `count`, elimina
/// los últimos `count` elementos.
/// Si la clave no existe, retorna `nil`. Si existe y `count` es mayor a 1 retorna un array con los elementos eliminados.
/// Si existe y no recibe el parámetro `count`, devuelve un bulkstring con el valor del último elemento.
/// Ante un error inesperado, devuelve Error `Invalid request`.
///
/// # Ejemplos
///
/// ```
/// use proyecto_taller_1::services::commands::command_list;
/// use proyecto_taller_1::services::utils::resp_type::RespType;
/// use proyecto_taller_1::domain::implementations::database::Database;
/// use std::sync::{Arc, RwLock};
/// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
///
/// let db = Database::new("dummy_db_rpop_command.csv".to_string());
/// let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItem::new_now(
/// ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string(), "melon".to_string(), "ciruela".to_string()]),
/// KeyAccessTime::Persistent
/// ));
///
/// let res = command_list::rpop(&vec![
/// RespType::RBulkString("RPOP".to_string()),
/// RespType::RBulkString("frutas".to_string()),
/// RespType::RBulkString("3".to_string())
/// ], &database);
///
/// match res {
///     RespType::RArray(frutas_eliminadas) => {
///     assert_eq!(frutas_eliminadas, vec![RespType::RBulkString("ciruela".to_string()),RespType::RBulkString("melon".to_string()), RespType::RBulkString("sandia".to_string())]) }
///     _ => assert!(false)
/// }
///
/// let _ = std::fs::remove_file("dummy_db_rpop_command.csv");
/// ```
pub fn rpop(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut db = database.write().unwrap();
    if let RespType::RBulkString(key) = &cmd[1] {
        if cmd.len() == 3 {
            if let RespType::RBulkString(cantidad) = &cmd[2] {
                let popped_elements =
                    db.rpop_elements_from_list(key, cantidad.parse::<usize>().unwrap());
                if let Some(popped) = popped_elements {
                    let mut p = Vec::new();
                    popped.iter().for_each(|element| {
                        p.push(RespType::RBulkString(element.to_string()));
                    });
                    return RespType::RArray(p);
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
/// # Ejemplos
///
/// ```
/// use proyecto_taller_1::services::commands::command_list;
/// use proyecto_taller_1::services::utils::resp_type::RespType;
/// use proyecto_taller_1::domain::implementations::database::Database;
/// use std::sync::{Arc, RwLock};
/// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
///
/// let db = Database::new("dummy_db_rpushx_command.csv".to_string());
/// let mut database = Arc::new(RwLock::new(db));
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItem::new_now(
/// ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()]),
/// KeyAccessTime::Persistent
/// ));
///
/// let res = command_list::rpushx(&vec![
/// RespType::RBulkString("RPUSHX".to_string()),
/// RespType::RBulkString("frutas".to_string()),
/// RespType::RBulkString("melon".to_string()),
/// RespType::RBulkString("sandia".to_string())
/// ], &database);
///
/// match res {
///     RespType::RInteger(len) => {
///     assert_eq!(len, 5) }
///     _ => assert!(false)
/// }
///
/// let _ = std::fs::remove_file("dummy_db_rpushx_command.csv");
/// ```
pub fn rpushx(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut new_database = database.write().unwrap();
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

//-------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------
//----------------------------------------FUNCIONES ADICIONALES------------------------------------------------
//-------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------

// pub fn pop_elements_from_db(
//     cantidad: usize,
//     key: String,
//     mut old_vec: Vec<String>,
//     mut database: RwLockWriteGuard<Database>,
// ) -> Vec<RespType> {
//     let mut vec_aux = vec![];
//     for _n in 0..cantidad {
//         let current_element = old_vec.pop().unwrap().to_string();
//         vec_aux.push(RespType::RBulkString(current_element));
//     }
//     let mut vec_to_stored = vec![];
//     for elemento in &vec_aux {
//         if let RespType::RBulkString(elem) = elemento {
//             vec_to_stored.push(elem.to_string());
//         };
//     };
// }
