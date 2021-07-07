use crate::domain::entities::key_value_item::{KeyAccessTime, ValueTimeItem, ValueType};
use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
//use std::collections::HashMap;
//use std::str::FromStr;
use std::sync::{Arc, RwLock};
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
                let vt_item = ValueTimeItem::new(
                    ValueType::ListType(vec_aux),
                    KeyAccessTime::Volatile(4234234),
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
            let vt_item = ValueTimeItem::new(
                ValueType::ListType(vec_aux),
                KeyAccessTime::Volatile(4234234),
            );
            new_database.add(key.to_string(), vt_item);
            RespType::RBulkString(vec_len.to_string())
        }
    } else {
        RespType::RBulkString("empty request".to_string())
    }
}
