use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Recibe un comando cmd de tipo &[RespType] y la base de datos database dentro de un RwLock
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

/// Recibe un comando cmd de tipo &[RespType] y la base de datos database dentro de un RwLock
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

/// Recibe un comando de tipo &[RespType]
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

/// Recibe un comando cmd de tipo &[RespType] y la base de datos database dentro de un RwLock
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

/// Recibe un comando cmd de tipo &[RespType] y la base de datos database dentro de un RwLock
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

/// Recibe un comando cmd de tipo &[RespType] y la base de datos database dentro de un RwLock
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

// pub fn sort (_cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
//     let database_lock = database.read().unwrap();//.get_all_values();
//     let values  = database_lock.get_all_values();
//     let vec_of_values  =*values;
//     println!("{:?}",vec_of_values);

//     RespType::RBulkString("hi".to_string())
// }

pub fn true_sort(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    //-----------------------
    //A hashpam is created to store all info about the SORT operation
    let mut aux_hash_map = HashMap::new();
    let keys = vec!["BY", "LIMIT", "GET", "ASC", "DESC", "ALPHA", "STORE"];
    let mut current_position;
    for key in keys {
        current_position = cmd
            .iter()
            .position(|x| x == &RespType::RBulkString(key.to_string()));
        if current_position != None {
            if (key == "ASC") || (key == "DESC") || (key == "ALPHA") {
                // || (key == "BY") {
                aux_hash_map.insert(key.to_string(), &RespType::RInteger(1)); // &cmd[current_position +1]);
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

    let mut vector = Vec::new();
    if let RespType::RBulkString(current_key) = &cmd[1] {
        let mut sorted_list: Vec<&String> = Vec::new();
        let mut auxiliary_vec = Vec::new();
        //let mut auxiliary_vec_2 = Vec::new();
        let database_lock = database.read().unwrap();
        //aca atrapo la lista que viene en la instruccion.. es una clave o un conjunto de claves
        let current_list = database_lock
            .search_item_by_key(current_key.to_string())
            .unwrap();
        if aux_hash_map.contains_key("BY") {
            if let RespType::RBulkString(pat) = aux_hash_map.get("BY").unwrap() {
                let elements = current_list.get_value_version_2().unwrap();
                let mut vec_aux = Vec::new();
                for aux in elements {
                    vec_aux.push(aux.to_string());
                }
                let mut tuple_vector; //= Vec::new();
                tuple_vector = database_lock
                    .get_values_of_external_keys_that_match_a_pattern(vec_aux, pat)
                    .unwrap();
                //println!("{:?}",&tuple_vector);
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
                sorted_list = current_list.sort_descending().unwrap();
            } else {
                //ordeno ascendentemente
                sorted_list = current_list.sort().unwrap();
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
            // if (aux_hash_map.contains_key("BY")) {

            // }
        }

        sorted_list
            .into_iter()
            .for_each(|value| vector.push(RespType::RBulkString(value.to_string())));
        RespType::RArray(vector)
    } else {
        RespType::RBulkString("empty".to_string())
    }
}

//-----------------------
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
