use crate::domain::entities::key_value_item::{KeyAccessTime, ValueTimeItem, ValueType};
use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
//use std::collections::HashMap;
//use std::str::FromStr;
//use std::time::SystemTime;

///GRUPO [LIST]: guarda elementos nuevos a una lista. Si no existe, la crea. Si el tipo de dato de la *key*
/// no es de tipo "lista", devuelve un error. En caso de que la operacion sea exitosa, se devuelve la
/// cantidad de elementos guardados en esa key
//REVISAR EL KEY_ACCES_TIME!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
pub fn lpush(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut new_database = database.write().unwrap();
    let mut vec_aux = vec![];
    if let RespType::RBulkString(key) = &cmd[1] {
        //guardo el contenido de la request en un vector
        for n in cmd.iter().skip(2).rev() {
            if let RespType::RBulkString(value) = n {
                vec_aux.push(value.to_string());
            }
        }
        if new_database.key_exists(key.to_string()) {
            let existing_value_type = new_database.get_type_of_value(key.to_string());
            println!("{:?}", existing_value_type);
            if existing_value_type == *"list" {
                let old_value = new_database.search_item_by_key(key.to_string()).unwrap();
                let oldie = ValueTimeItem::get_value_version_2(old_value).unwrap();
                for old_element in oldie.iter() {
                    vec_aux.push(old_element.to_string());
                }
                let vec_len = &vec_aux.len();
                let vt_item = ValueTimeItem::new_now(
                    ValueType::ListType(vec_aux),
                    KeyAccessTime::Volatile(1635597198),
                );
                new_database.add(key.to_string(), vt_item);
                RespType::RBulkString(vec_len.to_string())
            } else {
                RespType::RBulkString(
                    "la clave guarda un valor cuyo tipo no es una lista".to_string(),
                )
            }
        } else {
            let vec_len = &vec_aux.len();
            let vt_item = ValueTimeItem::new_now(
                ValueType::ListType(vec_aux),
                KeyAccessTime::Volatile(1635597199),
            );
            new_database.add(key.to_string(), vt_item);
            RespType::RBulkString(vec_len.to_string())
        }
    } else {
        RespType::RBulkString("empty request".to_string())
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
