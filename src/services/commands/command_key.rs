// use crate::domain::entities::key_value_item::ValueTimeItem;
// use crate::domain::entities::key_value_item::ValueType;
// use regex::Regex;
//use external regex::Regex;
// use crate::domain::entities::config::Config;
// use crate::domain::entities::message::WorkerMessage;
use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// use std::io::{Error, ErrorKind};
// use std::{net::SocketAddr, sync::mpsc::Sender};

pub fn del(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut n_key_deleted = 0;
    if cmd.len() == 1 {
        RespType::RInteger(n_key_deleted)
    } else {
        for n in cmd.iter().skip(1) {
            if let RespType::RBulkString(current_key) = n {
                let mut new_database = database.write().unwrap();
                new_database.delete_key(current_key.to_string());
                n_key_deleted += 1;
            }
        }
        RespType::RInteger(n_key_deleted)
    }
}

pub fn copy(
    database: &Arc<RwLock<Database>>,
    source: String,
    destination: String,
    replace: bool,
) -> Option<()> {
    if let Ok(write_guard) = database.write() {
        let mut db = write_guard;
        return db.copy(source, destination, replace);
    }
    None
}

pub fn exists(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut key_found = 0;
    if cmd.len() > 1 {
        for n in cmd.iter().skip(1) {
            if let RespType::RBulkString(actual_key) = n {
                if let Some(_key) = database
                    .read()
                    .unwrap()
                    .search_item_by_key(actual_key.to_string())
                {
                    key_found = 1;
                }
            }
        }
    }
    RespType::RInteger(key_found)
}

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

pub fn rename(cmd: &[RespType], database: &Arc<RwLock<Database>>) -> RespType {
    let mut message = "key not found".to_string();
    if cmd.len() > 1 {
        if let RespType::RBulkString(current_key) = &cmd[1] {
            let mut new_database = database.write().unwrap();
            if let RespType::RBulkString(new_key) = &cmd[2] {
                if new_database.rename_key(current_key.to_string(), new_key.to_string()) {
                    message = "key found and renamed succesfully".to_string();
                }
            }
        }
    }
    RespType::RBulkString(message)
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
        return RespType::RArray(vector);
    } else {
        return RespType::RBulkString("empty".to_string());
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
