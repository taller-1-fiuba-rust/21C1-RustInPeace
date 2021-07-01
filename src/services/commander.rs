use super::utils::resp_type::RespType;
use crate::domain::entities::key_value_item::ValueTimeItem; //, ValueType};
use crate::domain::implementations::database::Database;
use crate::{
    domain::entities::{config::Config, message::WorkerMessage},
    services::commands::command_key,
    services::commands::command_server,
    services::commands::command_string,
};
#[allow(unused)]
use std::fs::File;
use std::{
    net::SocketAddr,
    sync::{mpsc::Sender, Arc, RwLock},
};

/// Recibe una operacion operation de tipo RespType, un sender tx de mensajes de tipo WorkerMessage, la dirección del cliente addrs de tipo SocketAddrs
/// la base de datos database dentro de un RwLock y la configuración config dentro de un RwLock
/// Lee la primera palabra de la operación para disparar la acción que corresponda.
/// Devuelve un Option de tipo RespType con la respuesta que se le devolverá al cliente.
pub fn handle_command(
    operation: RespType,
    tx: &Sender<WorkerMessage>,
    addrs: SocketAddr,
    database: &Arc<RwLock<Database>>,
    config: &Arc<RwLock<Config>>,
) -> Option<RespType> {
    if let RespType::RArray(array) = operation {
        if let RespType::RBulkString(actual_command) = &array[0] {
            match actual_command.as_str() {
                "monitor" => {
                    command_server::monitor(&tx, &addrs);
                    // match self.operations.get(&addrs.to_string()) {
                    // Some(operations) => {
                    //     let last_ops = operations.get_operations();
                    //     command_server::monitor(last_ops);
                    // }
                    // None => println!("Client doesnt exist"),
                    return None;
                }
                "info" => {
                    let infor_requiered = command_server::info(&array);
                    println!("{:?}", infor_requiered);
                    return None;
                }
                "config" => {
                    if let RespType::RBulkString(instruction) = &array[1] {
                        match instruction.as_str() {
                            "get" => {
                                return Some(command_server::config_get(config, &array[2]));
                            }
                            "set" => {
                                return Some(command_server::config_set(
                                    config, &array[2], &array[3],
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                "dbsize" => {
                    return Some(command_server::dbsize(&database));
                }
                "flushdb" => {
                    return Some(command_server::flushdb(database));
                }
                "copy" => {
                    return Some(command_key::copy(&array, database));
                }
                "del" => {
                    return Some(command_key::del(&array, database));
                }
                "exists" => {
                    return Some(command_key::exists(&array, database));
                }
                "persist" => {
                    return Some(command_key::persist(&array, database));
                }
                "rename" => {
                    return Some(command_key::rename(&array, database));
                }
                "expire" => {
                    return Some(command_key::expire(&array, database));
                }
                "expireat" => {
                    return Some(command_key::expireat(&array, database));
                }
                "sort" => {
                    return Some(command_key::sort(&array, database));
                    //let sorted_list = command_key::sort(&array, database);
                    //println!("{:?}", sorted_list)
                }
                "keys" => return Some(command_key::keys(&array, database)),
                "touch" => return Some(command_key::keys(&array, database)),
                "type" => {
                    let tipo = command_key::get_type(&array, database);
                    println!("{:?}", tipo);
                    //return Some (command_key::get_type(&array, database))
                }
                "append" => {
                    return Some(command_string::append(&array, database));
                }
                "decrby" => {
                    return Some(command_string::decrby(&array, database));
                }
                "get" => {
                    return Some(command_string::get(&array, database));
                }
                "getdel" => {
                    return Some(command_string::getdel(&array, database));
                }
                "getset" => {
                    return Some(command_string::getset(&array, database));
                }
                "incrby" => {
                    return Some(command_string::incrby(&array, database));
                }
                "strlen" => {
                    return Some(command_string::strlen(&array, database));
                }
                "ttl" => {
                    return Some(command_key::get_ttl(&array, database));
                }
                _ => {}
            }
        }
    }
    None
}

pub fn load_data_in_db(database: &Arc<RwLock<Database>>, key: String, value: ValueTimeItem) {
    if let Ok(write_guard) = database.write() {
        let mut db = write_guard;
        db.add(key, value)
    }
}

#[test]
fn test_001_returns_dbsize() {
    use std::net::{IpAddr, Ipv4Addr};
    let _file = File::create("filename_dbsize".to_string());
    let config = Config::new(String::from("./src/redis.conf"));
    let db = Database::new("filename_dbsize".to_string());
    let database = Arc::new(RwLock::new(db));
    let conf = Arc::new(RwLock::new(config));
    let operation = RespType::RArray(vec![RespType::RBulkString("dbsize".to_string())]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    handle_command(operation, &tx, addrs, &database, &conf);
    let _ = std::fs::remove_file("filename_dbsize".to_string());
}

#[test]
fn test_002_shows_server_info() {
    use std::net::{IpAddr, Ipv4Addr};
    let db = Database::new("filename".to_string());
    let database = Arc::new(RwLock::new(db));
    let config = Config::new(String::from("./src/redis.conf"));
    let conf = Arc::new(RwLock::new(config));
    let operation = RespType::RArray(vec![
        RespType::RBulkString("info".to_string()),
        RespType::RBulkString("server".to_string()),
    ]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

    handle_command(operation, &tx, addrs, &database, &conf);
    let _ = std::fs::remove_file("filename".to_string());
}

#[test]
fn test_003_cleans_db_items() {
    use std::net::{IpAddr, Ipv4Addr};
    let db = Database::new("filename_3".to_string());
    let database = Arc::new(RwLock::new(db));
    let operation = RespType::RArray(vec![RespType::RBulkString("flushdb".to_string())]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let config = Config::new(String::from("./src/redis.conf"));
    let conf = Arc::new(RwLock::new(config));
    handle_command(operation, &tx, addrs, &database, &conf);
    let operation_check_dbsize =
        RespType::RArray(vec![RespType::RBulkString("dbsize".to_string())]);
    handle_command(operation_check_dbsize, &tx, addrs, &database, &conf);
    let _ = std::fs::remove_file("filename_3".to_string());
}

#[test]
fn test_004_deletes_a_key_from_db() {
    use std::net::{IpAddr, Ipv4Addr};
    let db = Database::new("filename_4".to_string());
    let database = Arc::new(RwLock::new(db));
    let operation = RespType::RArray(vec![
        RespType::RBulkString("del".to_string()),
        RespType::RBulkString("clave_1".to_string()),
    ]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let config = Config::new(String::from("./src/redis.conf"));
    let conf = Arc::new(RwLock::new(config));
    handle_command(operation, &tx, addrs, &database, &conf);
    let operation_check_dbsize =
        RespType::RArray(vec![RespType::RBulkString("dbsize".to_string())]);
    handle_command(operation_check_dbsize, &tx, addrs, &database, &conf);
    let _ = std::fs::remove_file("filename_4".to_string());
}

#[test]
fn test_005_check_if_key_exists_throws_zero() {
    use std::net::{IpAddr, Ipv4Addr};
    let db = Database::new("filename_5".to_string());
    let database = Arc::new(RwLock::new(db));
    let operation = RespType::RArray(vec![
        RespType::RBulkString("exists".to_string()),
        RespType::RBulkString("clave_3".to_string()),
    ]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let config = Config::new(String::from("./src/redis.conf"));
    let conf = Arc::new(RwLock::new(config));
    handle_command(operation, &tx, addrs, &database, &conf);
    let _ = std::fs::remove_file("filename_5".to_string());
}

#[test]
fn test_006_check_if_key_exists_throws_one() {
    use std::net::{IpAddr, Ipv4Addr};
    let db = Database::new("filename_6".to_string());
    let database = Arc::new(RwLock::new(db));
    let operation = RespType::RArray(vec![
        RespType::RBulkString("exists".to_string()),
        RespType::RBulkString("clave_1".to_string()),
    ]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let config = Config::new(String::from("./src/redis.conf"));
    let conf = Arc::new(RwLock::new(config));
    handle_command(operation, &tx, addrs, &database, &conf);
    let _ = std::fs::remove_file("filename_6".to_string());
}

#[test]
fn test_007_sort_ascending() {
    use crate::domain::entities::key_value_item::KeyAccessTime;
    use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};

    use std::net::{IpAddr, Ipv4Addr};
    let db = Database::new("filename_7".to_string());
    let database = Arc::new(RwLock::new(db));
    //se rellena la database
    let vt_1 = ValueTimeItem::new_now(
        ValueType::ListType(vec![
            "15".to_string(),
            "18".to_string(),
            "12".to_string(),
            "54".to_string(),
            "22".to_string(),
            "45".to_string(),
        ]),
        //value: ValueType::StringType("1".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_2 = ValueTimeItem::new_now(
        ValueType::StringType("2".to_string()),
        KeyAccessTime::Volatile(0),
    );
    load_data_in_db(&database, "edades_amigos".to_string(), vt_1);
    load_data_in_db(&database, "edades_familiares".to_string(), vt_2);
    //se relleno la database
    let operation = RespType::RArray(vec![
        RespType::RBulkString("sort".to_string()),
        RespType::RBulkString("edades_amigos".to_string()),
    ]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let config = Config::new(String::from("./src/redis.conf"));
    let conf = Arc::new(RwLock::new(config));
    handle_command(operation, &tx, addrs, &database, &conf);
    let _ = std::fs::remove_file("filename_7".to_string());
}

#[test]
fn test_008_sort_descending() {
    use crate::domain::entities::key_value_item::KeyAccessTime;
    use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};

    use std::net::{IpAddr, Ipv4Addr};
    let db = Database::new("filename_008".to_string());
    let database = Arc::new(RwLock::new(db));
    //se rellena la database
    let vt_1 = ValueTimeItem::new_now(
        ValueType::ListType(vec![
            "15".to_string(),
            "18".to_string(),
            "12".to_string(),
            "54".to_string(),
            "22".to_string(),
            "45".to_string(),
        ]),
        //value: ValueType::StringType("1".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_2 = ValueTimeItem::new_now(
        ValueType::StringType("2".to_string()),
        KeyAccessTime::Volatile(0),
    );
    load_data_in_db(&database, "edades_amigos".to_string(), vt_1);
    load_data_in_db(&database, "edades_familiares".to_string(), vt_2);
    //se relleno la database
    let operation = RespType::RArray(vec![
        RespType::RBulkString("sort".to_string()),
        RespType::RBulkString("edades_amigos".to_string()),
        RespType::RBulkString("DESC".to_string()),
    ]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let config = Config::new(String::from("./src/redis.conf"));
    let conf = Arc::new(RwLock::new(config));
    handle_command(operation, &tx, addrs, &database, &conf);
    let _removed = std::fs::remove_file("filename_008".to_string());
}

#[test]
fn test_009_sort_ascending_first_4_elements() {
    use crate::domain::entities::key_value_item::KeyAccessTime;
    use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};

    use std::net::{IpAddr, Ipv4Addr};
    let db = Database::new("filename_101".to_string());
    let database = Arc::new(RwLock::new(db));
    //se rellena la database
    let vt_1 = ValueTimeItem::new_now(
        ValueType::ListType(vec![
            "15".to_string(),
            "18".to_string(),
            "12".to_string(),
            "54".to_string(),
            "22".to_string(),
            "45".to_string(),
        ]),
        //value: ValueType::StringType("1".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_2 = ValueTimeItem::new_now(
        ValueType::StringType("2".to_string()),
        KeyAccessTime::Volatile(0),
    );
    load_data_in_db(&database, "edades_amigos".to_string(), vt_1);
    load_data_in_db(&database, "edades_familiares".to_string(), vt_2);
    //se relleno la database
    let operation = RespType::RArray(vec![
        RespType::RBulkString("sort".to_string()),
        RespType::RBulkString("edades_amigos".to_string()),
        RespType::RBulkString("LIMIT".to_string()),
        RespType::RBulkString("0".to_string()),
        RespType::RBulkString("4".to_string()),
    ]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let config = Config::new(String::from("./src/redis.conf"));
    let conf = Arc::new(RwLock::new(config));
    handle_command(operation, &tx, addrs, &database, &conf);
    let _removed = std::fs::remove_file("filename_101".to_string());
}

#[test]
fn test_010_sort_descending_first_4_elements() {
    use crate::domain::entities::key_value_item::KeyAccessTime;
    use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};

    use std::net::{IpAddr, Ipv4Addr};
    let db = Database::new("filename_701".to_string());
    let database = Arc::new(RwLock::new(db));
    //se rellena la database
    let vt_1 = ValueTimeItem::new_now(
        ValueType::ListType(vec![
            "15".to_string(),
            "18".to_string(),
            "12".to_string(),
            "54".to_string(),
            "22".to_string(),
            "45".to_string(),
        ]),
        //value: ValueType::StringType("1".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_2 = ValueTimeItem::new_now(
        ValueType::StringType("2".to_string()),
        KeyAccessTime::Volatile(0),
    );
    load_data_in_db(&database, "edades_amigos".to_string(), vt_1);
    load_data_in_db(&database, "edades_familiares".to_string(), vt_2);
    //se relleno la database
    let operation = RespType::RArray(vec![
        RespType::RBulkString("sort".to_string()),
        RespType::RBulkString("edades_amigos".to_string()),
        RespType::RBulkString("LIMIT".to_string()),
        RespType::RBulkString("0".to_string()),
        RespType::RBulkString("4".to_string()),
        RespType::RBulkString("DESC".to_string()),
    ]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let config = Config::new(String::from("./src/redis.conf"));
    let conf = Arc::new(RwLock::new(config));
    handle_command(operation, &tx, addrs, &database, &conf);
    let _removed = std::fs::remove_file("filename_701".to_string());
}

#[test]
fn test_011_sort_by_external_key_value_using_pattern_ascending() {
    use crate::domain::entities::key_value_item::KeyAccessTime;
    use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};

    use std::net::{IpAddr, Ipv4Addr};
    let db = Database::new("filename_706".to_string());
    let database = Arc::new(RwLock::new(db));
    //se rellena la database
    let vt_1 = ValueTimeItem::new_now(
        // value: ValueType::ListType(vec!["15".to_string()]),
        ValueType::StringType("10".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_2 = ValueTimeItem::new_now(
        ValueType::StringType("20".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_3 = ValueTimeItem::new_now(
        // value: ValueType::ListType(vec!["11".to_string()]),
        ValueType::StringType("10".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_4 = ValueTimeItem::new_now(
        ValueType::StringType("40".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_5 = ValueTimeItem::new_now(
        // value: ValueType::ListType(vec!["1".to_string()]),
        ValueType::StringType("50".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_6 = ValueTimeItem::new_now(
        ValueType::StringType("60".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_7 = ValueTimeItem::new_now(
        ValueType::ListType(vec![
            "ignacio".to_string(),
            "pepo".to_string(),
            "silvina".to_string(),
            "lucila".to_string(),
        ]),
        KeyAccessTime::Volatile(0),
    );
    load_data_in_db(&database, "edad_juana".to_string(), vt_1);
    load_data_in_db(&database, "edad_silvina".to_string(), vt_2);
    load_data_in_db(&database, "edad_ignacio".to_string(), vt_3);
    load_data_in_db(&database, "edad_pepo".to_string(), vt_4);
    load_data_in_db(&database, "juana".to_string(), vt_5);
    load_data_in_db(&database, "lucila_edad".to_string(), vt_6);
    load_data_in_db(&database, "familiares".to_string(), vt_7);

    //se relleno la database
    let operation = RespType::RArray(vec![
        RespType::RBulkString("sort".to_string()),
        RespType::RBulkString("familiares".to_string()),
        RespType::RBulkString("BY".to_string()),
        RespType::RBulkString("edad_".to_string()),
    ]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let config = Config::new(String::from("./src/redis.conf"));
    let conf = Arc::new(RwLock::new(config));
    handle_command(operation, &tx, addrs, &database, &conf);
    let _ = std::fs::remove_file("filename_706".to_string());
}

#[test]
fn test_012_sort_by_external_key_value_using_pattern_descending() {
    use crate::domain::entities::key_value_item::KeyAccessTime;
    use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};

    use std::net::{IpAddr, Ipv4Addr};
    let db = Database::new("filename_7".to_string());
    let database = Arc::new(RwLock::new(db));
    //se rellena la database
    let vt_1 = ValueTimeItem::new_now(
        // value: ValueType::ListType(vec!["15".to_string()]),
        ValueType::StringType("10".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_2 = ValueTimeItem::new_now(
        ValueType::StringType("20".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_3 = ValueTimeItem::new_now(
        // value: ValueType::ListType(vec!["11".to_string()]),
        ValueType::StringType("30".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_4 = ValueTimeItem::new_now(
        ValueType::StringType("40".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_5 = ValueTimeItem::new_now(
        // value: ValueType::ListType(vec!["1".to_string()]),
        ValueType::StringType("50".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_6 = ValueTimeItem::new_now(
        ValueType::StringType("60".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_7 = ValueTimeItem::new_now(
        ValueType::ListType(vec![
            "ignacio".to_string(),
            "pepo".to_string(),
            "silvina".to_string(),
            "lucila".to_string(),
        ]),
        KeyAccessTime::Volatile(0),
    );
    load_data_in_db(&database, "edad_juana".to_string(), vt_1);
    load_data_in_db(&database, "edad_silvina".to_string(), vt_2);
    load_data_in_db(&database, "edad_ignacio".to_string(), vt_3);
    load_data_in_db(&database, "edad_pepo".to_string(), vt_4);
    load_data_in_db(&database, "juana".to_string(), vt_5);
    load_data_in_db(&database, "lucila_edad".to_string(), vt_6);
    load_data_in_db(&database, "familiares".to_string(), vt_7);

    //se relleno la database
    let operation = RespType::RArray(vec![
        RespType::RBulkString("sort".to_string()),
        RespType::RBulkString("familiares".to_string()),
        RespType::RBulkString("BY".to_string()),
        RespType::RBulkString("edad_".to_string()),
        RespType::RBulkString("DESC".to_string()),
    ]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let config = Config::new(String::from("./src/redis.conf"));
    let conf = Arc::new(RwLock::new(config));
    handle_command(operation, &tx, addrs, &database, &conf);
    let _removed = std::fs::remove_file("filename_7".to_string());
}

#[test]
fn test_013_gets_value_type_list() {
    use crate::domain::entities::key_value_item::KeyAccessTime;
    use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};

    use std::net::{IpAddr, Ipv4Addr};
    let _file = File::create("filename_13".to_string());
    let db = Database::new("filename_13".to_string());
    let database = Arc::new(RwLock::new(db));
    //se rellena la database
    let vt_1 = ValueTimeItem::new_now(
        // value: ValueType::ListType(vec!["15".to_string()]),
        ValueType::StringType("10".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_2 = ValueTimeItem::new_now(
        ValueType::StringType("20".to_string()),
        KeyAccessTime::Volatile(0),
    );

    let vt_7 = ValueTimeItem::new_now(
        ValueType::ListType(vec![
            "ignacio".to_string(),
            "pepo".to_string(),
            "silvina".to_string(),
            "lucila".to_string(),
        ]),
        KeyAccessTime::Volatile(0),
    );
    load_data_in_db(&database, "edad_juana".to_string(), vt_1);
    load_data_in_db(&database, "edad_silvina".to_string(), vt_2);
    load_data_in_db(&database, "familiares".to_string(), vt_7);

    //se relleno la database
    let operation = RespType::RArray(vec![
        RespType::RBulkString("type".to_string()),
        RespType::RBulkString("familiares".to_string()),
    ]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let config = Config::new(String::from("./src/redis.conf"));
    let conf = Arc::new(RwLock::new(config));
    handle_command(operation, &tx, addrs, &database, &conf);
    let _ = std::fs::remove_file("filename_13".to_string());
}

#[test]
fn test_014_gets_value_type_string() {
    use crate::domain::entities::key_value_item::KeyAccessTime;
    use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};

    use std::net::{IpAddr, Ipv4Addr};
    let db = Database::new("filename_014".to_string());
    let database = Arc::new(RwLock::new(db));
    //se rellena la database
    let vt_1 = ValueTimeItem::new_now(
        // value: ValueType::ListType(vec!["15".to_string()]),
        ValueType::StringType("10".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let vt_2 = ValueTimeItem::new_now(
        ValueType::StringType("20".to_string()),
        KeyAccessTime::Volatile(0),
    );

    let vt_7 = ValueTimeItem::new_now(
        ValueType::ListType(vec![
            "ignacio".to_string(),
            "pepo".to_string(),
            "silvina".to_string(),
            "lucila".to_string(),
        ]),
        KeyAccessTime::Volatile(0),
    );
    load_data_in_db(&database, "edad_juana".to_string(), vt_1);
    load_data_in_db(&database, "edad_silvina".to_string(), vt_2);
    load_data_in_db(&database, "familiares".to_string(), vt_7);

    //se relleno la database
    let operation = RespType::RArray(vec![
        RespType::RBulkString("type".to_string()),
        RespType::RBulkString("edad_juana".to_string()),
    ]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let config = Config::new(String::from("./src/redis.conf"));
    let conf = Arc::new(RwLock::new(config));
    handle_command(operation, &tx, addrs, &database, &conf);
    let _ = std::fs::remove_file("filename_014".to_string());
}
