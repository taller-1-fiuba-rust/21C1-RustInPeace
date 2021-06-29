use crate::domain::entities::key_value_item::KeyAccessTime;
use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// GRUPO [KEYS]:Recibe un comando cmd de tipo &[RespType] y la base de datos database dentro de un RwLock
/// Elimina las claves recibidas en el comando
/// Devuelve la cantidad de claves eliminadas en un tipo de dato RespType
pub fn del(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut n_key_deleted = 0;

    for n in cmd.iter().skip(1) {
        if let RespType::RBulkString(current_key) = n {
            let mut new_database = database.write().unwrap();
            new_database.delete_key(current_key.to_string());
            n_key_deleted += 1;
        }
    }

    RespType::RInteger(n_key_deleted)
}

/// GRUPO [KEYS]:Recibe un comando cmd de tipo &[RespType] y la base de datos database dentro de un RwLock
/// Extra del comando una clave fuente y una clave destino, copia el valor guardado en la clave fuente a la clave destino.
/// Si el comando contiene el parametro "replace", en el caso de que ya exista una clave con el mismo nombre que la clave destino,
/// se reemplaza su valor por el valor de la clave fuente y se devuelve un entero 1. En el caso que la clave destino ya exista pero no se incluya
/// el parametro "replace", entonces no se copia el valor y se devuelve un entero 0.
/// Si no existe la clave, se crea, se guarda una copia del valor que guarda la clave fuente y se devuelve un entero 1.
/// Ante algun error se devuelve un entero 0.
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

/// GRUPO [KEYS]:Recibe un comando de tipo &[RespType]
/// Verifica que si el ultimo parametro es "replace". Si lo es, devuelve true, sino devuelve false.
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

/// GRUPO [KEYS]:Recibe un comando cmd de tipo &[RespType] y la base de datos database dentro de un RwLock
/// Verifica si las claves extraidas del comando existen en la base de datos o no.
/// Devuelve la cantidad de claves encontradas.
pub fn exists(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut key_found = 0;
    if cmd.len() > 1 {
        for n in cmd.iter().skip(1) {
            if let RespType::RBulkString(current_key) = n {
                if database.read().unwrap().key_exists(current_key.to_string()) {
                    key_found += 1;
                }
            }
        }
    }
    RespType::RInteger(key_found)
}

/// GRUPO [KEYS]:Recibe un comando cmd de tipo &[RespType] y la base de datos database dentro de un RwLock
/// Extrae una clave key del comando, intenta cambiar el tipo de clave de volatir a persistente.
/// En caso de exito devuelve un 1, si falla devuelve un 0.
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

/// GRUPO [KEYS]:Recibe un comando cmd de tipo &[RespType] y la base de datos database dentro de un RwLock
/// Extrae una clave current_key y una clave new_key del comando.
/// Renombre a current_key por new_key y devuelve un BulkString con el mensaje "OK"
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
/// GRUPO [KEYS]:  Configura un tiempo de expiracion sobre una clave (la clave se dice que
/// es volatil). Luego de ese tiempo de expiracion, la clave es automaticamente eliminada.
/// El comando recibe 2 parámetros: la key y el tiempo de expiración (en segundos)
/// Devuele 1 si pudo ser seteado, o 0 en caso contrario.
pub fn expire(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() != 3 {
    } else if let RespType::RBulkString(key) = &cmd[1] {
        let mut db = database.write().unwrap();
        if let RespType::RBulkString(timeout) = &cmd[2] {
            let result = db.expire_key(key, timeout);
            if result {
                return RespType::RInteger(1);
            } else {
                return RespType::RInteger(0);
            }
        }
    }
    RespType::RInteger(0)
}

/// GRUPO [KEYS]:Recibe un comando **cmd** de tipo &[RespType] y la base de datos **database** dentro de un RwLock.
///Ordena una lista alojada como **value** de una **key** de distitnas formas:
/// ascendente y descendentemente y puede limitarse la cantidad de elementos a ordenar. Ademas,
/// permite ordenar utilizando claves externas y sus valores asociados
pub fn sort(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    //A hashpam is created to store all info about the SORT operation
    let aux_hash_map = generate_hashmap(cmd);
    let mut vector = Vec::new();
    if let RespType::RBulkString(current_key) = &cmd[1] {
        let mut sorted_list: Vec<&String> = Vec::new();
        let mut auxiliary_vec = Vec::new();
        //let mut auxiliary_vec_2 = Vec::new();
        let database_lock = database.read().unwrap();
        //aca atrapo "myList", que (si existe) es una key en la database
        let my_list_value = database_lock
            .search_item_by_key(current_key.to_string())
            .unwrap();
        if aux_hash_map.contains_key("BY") {
            if let RespType::RBulkString(pat) = aux_hash_map.get("BY").unwrap() {
                let elements = my_list_value.get_value_version_2().unwrap();
                //genero un vec_aux para guardar los "values" guardados en myList
                //(la que se pide ordenar) como String. Facilita la comparacion para
                //hallar el patron solicitado
                let mut vec_resptype_to_string = Vec::new();
                for aux in elements {
                    vec_resptype_to_string.push(aux.to_string());
                }
                let mut tuple_vector; //= Vec::new();
                tuple_vector = database_lock
                    .get_values_of_external_keys_that_match_a_pattern(vec_resptype_to_string, pat)
                    .unwrap();
                tuple_vector.sort_by_key(|k| k.1.clone());
                for val in tuple_vector {
                    auxiliary_vec.push(val.0.clone());
                }
                for j in &auxiliary_vec {
                    sorted_list.push(j)
                }
                if aux_hash_map.contains_key("DESC") {
                    sorted_list.reverse()
                }
                if (aux_hash_map.contains_key("LOWER")) || (aux_hash_map.contains_key("UPPER")) {
                    if let RespType::RBulkString(lower_bound) = aux_hash_map.get("LOWER").unwrap() {
                        if let RespType::RBulkString(upper_bound) =
                            aux_hash_map.get("UPPER").unwrap()
                        {
                            let min = lower_bound.parse::<usize>().unwrap();
                            let max = upper_bound.parse::<usize>().unwrap();
                            sorted_list = sorted_list[min..max].to_vec();
                        }
                    }
                }
            }
        } else {
            if aux_hash_map.contains_key("DESC") {
                //ordeno descendentemente
                sorted_list = my_list_value.sort_descending().unwrap();
            } else {
                //ordeno ascendentemente
                sorted_list = my_list_value.sort().unwrap();
            }
            if (aux_hash_map.contains_key("LOWER")) || (aux_hash_map.contains_key("UPPER")) {
                if let RespType::RBulkString(lower_bound) = aux_hash_map.get("LOWER").unwrap() {
                    if let RespType::RBulkString(upper_bound) = aux_hash_map.get("UPPER").unwrap() {
                        let min = lower_bound.parse::<usize>().unwrap();
                        let max = upper_bound.parse::<usize>().unwrap();
                        sorted_list = sorted_list[min..max].to_vec();
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

pub fn touch(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut number_of_touched_keys = 0;
    let mut db = database.write().unwrap();
    if cmd.len() > 1 {
        for n in cmd.iter().skip(1) {
            if let RespType::RBulkString(current_key) = n {
                if db.key_exists(current_key.to_string()) {
                    db.reboot_time(current_key.to_string());
                    number_of_touched_keys += 1;
                }
            }
        }
    }
    RespType::RInteger(number_of_touched_keys)
}

pub fn get_type(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let db = database.read().unwrap();
    let mut tipo = String::from("");
    if let RespType::RBulkString(current_key) = &cmd[1] {
        if db.key_exists(current_key.to_string()) {
            tipo = db.get_type_of_value(current_key.to_string());
        }
    }
    RespType::RBulkString(tipo)
}

//--------------------------------------------------------------------
/// Permite generar un hashmap a partir de un grupo de claves hardcodeadas y asociarles un valor de existencia
fn generate_hashmap(cmd: &[RespType]) -> HashMap<String, &RespType> {
    let mut aux_hash_map = HashMap::new();
    let keys = vec!["BY", "LIMIT", "GET", "ASC", "DESC", "ALPHA", "STORE"];
    let mut current_position;
    for key in keys {
        current_position = cmd
            .iter()
            .position(|x| x == &RespType::RBulkString(key.to_string()));
        if current_position != None {
            if (key == "ASC") || (key == "DESC") || (key == "ALPHA") {
                aux_hash_map.insert(key.to_string(), &RespType::RInteger(1));
            } else if (key == "BY") || (key == "STORE") {
                aux_hash_map.insert(key.to_string(), &cmd[current_position.unwrap() + 1]);
            } else if key == "LIMIT" {
                aux_hash_map.insert("LOWER".to_string(), &cmd[current_position.unwrap() + 1]);
                aux_hash_map.insert("UPPER".to_string(), &cmd[current_position.unwrap() + 2]);
            }
            // } else if (key == "GET") {
            // }
        }
    }
    aux_hash_map
}
/// Retorna el tiempo que le queda a una clave para que se cumpla su timeout (en segundos)
/// En caso que no sea una clave volátil retorna (-1) y si no existe, retorna (-2)
pub fn get_ttl(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    if cmd.len() > 1 {
        if let RespType::RBulkString(key) = &cmd[1] {
            let db = database.write().unwrap();
            match db._get_items().get(key) {
                None => return RespType::RNegative(-2),
                Some(item) => {
                    return match item.get_timeout() {
                        KeyAccessTime::Volatile(timeout) => RespType::RInteger(*timeout as usize),
                        KeyAccessTime::Persistent => RespType::RInteger(0),
                    }
                }
            }
        }
    }
    RespType::RInteger(0)
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

//--------------------------------------------------------------------
//let mut vector = Vec::new();
// if aux_hash_map.contains_key("ALPHA"){
//     //ordeno alfabeticamente
// }
// if aux_hash_map.contains_key("DESC")

// if let RespType::RBulkString(current_key) = &cmd[1] {
//     let database_lock = database.read().unwrap();
//     let current_list = database_lock.search_item_by_key(current_key.to_string()).unwrap();
//     //let mut vector = Vec::new();
//     if cmd.len()>2 {
//         if let RespType::RBulkString(current_inst) = &cmd[2] {
//             if (current_inst.to_string() == "DESC") || ((current_inst.to_string() == "ALPHA")&&(&cmd[3]=="DESC") ){
//                 let sorted_list_desc = current_list.sort_descending().unwrap().into_iter();
//                 //let mut vector = Vec::new();
//                 sorted_list_desc.into_iter().for_each(|value| {
//                     vector.push(RespType::RBulkString(value.to_string()))
//                 });
//                 //return RespType::RArray(vector);
//             }

//             if current_inst.to_string() == "LIMIT" {
//                 if cmd.len()>4 {
//                     let min = &cmd[3];
//                     let max = &cmd[4];

//                     if let RespType::RBulkString(current_inst) = &cmd[5] {

//                     }

//                 } else {
//                     let sorted_list = current_list.sort().unwrap();
//                     let sorted_list_shorten = sorted_list[min..max].to_vec();
//                     sorted_list_shorten.into_iter().for_each(|value| {
//                         vector.push(RespType::RBulkString(value.to_string()))
//                     });
//                 }

//                 }

//             } else {

//             }
//         }
//     } else {
//         let sorted_list = current_list.sort().unwrap();
//         sorted_list.into_iter().for_each(|value| {
//             vector.push(RespType::RBulkString(value.to_string()))
//         });
//     }

// }
//     //tengo un ValueType (que puede ser List o Set - String no figura en el pryecto, hay que preg )
// //ValueType::ListType();
// RespType::RArray(vector)
//}

//     for n in cmd.iter().skip(1) {
//         //            key_in_db = database.search_by_key()
//         if let RespType::RBulkString(actual_key) = n {
//             if let Some(_key) = database.read().unwrap().search_item_by_key(actual_key) {
//                 key_found = 1;
//             }
//         }
//     }
// }
