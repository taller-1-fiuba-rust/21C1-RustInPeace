use crate::domain::entities::key_value_item::KeyAccessTime;
use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

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

/// GRUPO [KEYS]:  Configura un tiempo de expiracion sobre una clave (la clave se dice que
/// es volatil). Luego de ese tiempo de expiracion, la clave es automaticamente eliminada.
/// El comando recibe 2 parámetros: la key y el nuevo timestamp
/// Devuele 1 si pudo ser seteado, o 0 en caso contrario.
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
        if aux_hash_map.contains_key("by") {
            if let RespType::RBulkString(pat) = aux_hash_map.get("by").unwrap() {
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
            if aux_hash_map.contains_key("desc") {
                //ordeno descendentemente

                sorted_list = my_list_value.sort_descending().unwrap();
            } else {
                //ordeno ascendentemente
                sorted_list = my_list_value.sort().unwrap();
            }
            if (aux_hash_map.contains_key("lower")) || (aux_hash_map.contains_key("upper")) {
                if let RespType::RBulkString(lower_bound) = aux_hash_map.get("lower").unwrap() {
                    if let RespType::RBulkString(upper_bound) = aux_hash_map.get("upper").unwrap() {
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
/// use proyecto_taller_1::domain::implementations::database::Database;
/// use std::sync::{Arc, RwLock};
/// use proyecto_taller_1::domain::entities::key_value_item::{ValueType, ValueTimeItem, KeyAccessTime};
/// use std::time::SystemTime;
/// use proyecto_taller_1::services::utils::resp_type::RespType;
/// use proyecto_taller_1::services::commands::command_key;
///
/// // Agrego los datos en la base
///
/// let db = Database::new("dummy_db_doc_touch1.csv".to_string());
/// let mut database = Arc::new(RwLock::new(db));
///
/// let mut timeout_10seg = SystemTime::now()
///  .duration_since(SystemTime::UNIX_EPOCH)
///   .unwrap().as_secs();
/// timeout_10seg += 10;
///
/// database.write().unwrap().add("frutas".to_string(),ValueTimeItem::new_now(
/// ValueType::ListType(vec!["kiwi".to_string(),"pomelo".to_string(),"sandia".to_string()]),
/// KeyAccessTime::Persistent
/// ));
///
/// database.write().unwrap().add("verduras".to_string(),ValueTimeItem::new_now(
/// ValueType::ListType(vec!["acelga".to_string(),"cebolla".to_string(),"zanahoria".to_string()]),
/// KeyAccessTime::Volatile(timeout_10seg)
/// ));
///
/// //Ejecuto el comando con los parámetros necesarios:
/// let res = command_key::touch(&vec![
/// RespType::RBulkString("TOUCH".to_string()),
/// RespType::RBulkString("frutas".to_string()),
/// RespType::RBulkString("verduras".to_string())
/// ], &database);
///
/// match res {
///     RespType::RInteger(quantity) => {
///     assert_eq!(quantity, 2) }
///     _ => assert!(false)
/// }
///
/// let _ = std::fs::remove_file("dummy_db_doc_touch1.csv");
/// ```
/// 2. Itenta actualizar 2 `keys`donde una está expirada y la otra no existe en la database
/// ```
/// use proyecto_taller_1::domain::implementations::database::Database;
/// use std::sync::{Arc, RwLock};
/// use proyecto_taller_1::domain::entities::key_value_item::{ValueType, ValueTimeItem, KeyAccessTime};
/// use std::time::SystemTime;
/// use proyecto_taller_1::services::utils::resp_type::RespType;
/// use proyecto_taller_1::services::commands::command_key;
///
/// let db = Database::new("dummy_db_doc_touch2.csv".to_string());
/// let mut database = Arc::new(RwLock::new(db));
///
/// let timeout_now = SystemTime::now()
///  .duration_since(SystemTime::UNIX_EPOCH)
///   .unwrap().as_secs();
///
/// database.write().unwrap().add("verduras".to_string(),ValueTimeItem::new_now(
/// ValueType::ListType(vec!["acelga".to_string(),"cebolla".to_string(),"zanahoria".to_string()]),
/// KeyAccessTime::Volatile(timeout_now)
/// ));
///
/// //Ejecuto el comando con los parámetros necesarios:
/// let res = command_key::touch(&vec![
/// RespType::RBulkString("TOUCH".to_string()),
/// RespType::RBulkString("frutas".to_string()),
/// RespType::RBulkString("verduras".to_string())
/// ], &database);
///
/// match res {
///     RespType::RInteger(quantity) => {
///     assert_eq!(quantity, 0) }
///     _ => assert!(false)
/// }
///
/// let _ = std::fs::remove_file("dummy_db_doc_touch2.csv");
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

//--------------------------------------------------------------------
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

/// Retorna el tiempo que le queda a una clave para que se cumpla su timeout (en segundos)
/// En caso que no sea una clave volátil retorna (-1) y si no existe, retorna (-2)
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
