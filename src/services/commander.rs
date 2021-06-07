use super::utils::resp_type::RespType;
use crate::domain::implementations::database::Database;
use crate::{
    domain::entities::{config::Config, message::WorkerMessage},
    services::commands::command_key,
    services::commands::command_server,
};
use std::{
    net::SocketAddr,
    sync::{mpsc::Sender, Arc, RwLock},
};

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
                                command_server::config_get(config, &array[2]).unwrap();
                            }
                            "set" => {
                                command_server::config_set(config, &array[2], &array[3]).unwrap();
                            }
                            _ => {}
                        }
                    }
                    //implementar respuesta
                    return None;
                }
                "dbsize" => {
                    let db_size = command_server::dbsize(&database);
                    println!("database size: {:?}", db_size);
                    return Some(db_size);
                }
                "flushdb" => {
                    let erased = command_server::flushdb(database);
                    println!("{:?}", erased);
                    return None;
                }
                "copy" => {
                    if array.len() > 2 {
                        if let RespType::RBulkString(source) = &array[1] {
                            if let RespType::RBulkString(destination) = &array[2] {
                                if array.len() == 3 {
                                    let res = command_key::copy(
                                        database,
                                        String::from(source),
                                        String::from(destination),
                                        false,
                                    );
                                    if let Some(()) = res {
                                        return Some(RespType::RInteger(1));
                                    } else {
                                        return Some(RespType::RInteger(0));
                                    }
                                } else if array.len() == 4 {
                                    if let RespType::RBulkString(replace) = &array[3] {
                                        if replace == "replace" {
                                            let res = command_key::copy(
                                                database,
                                                String::from(source),
                                                String::from(destination),
                                                true,
                                            );
                                            if let Some(()) = res {
                                                return Some(RespType::RInteger(1));
                                            } else {
                                                return Some(RespType::RInteger(0));
                                            }
                                        }
                                    }
                                }
                            } else {
                                //
                            }
                        } else {
                            //
                        }
                    } else {
                        //
                    }
                    return None;
                }
                "del" => {
                    command_key::del(&array, database);
                }
                "exists" => {
                    let key_found = command_key::exists(&array, database);
                    println!("{:?}", key_found);
                }
                _ => {}
            }
        }
    }
    None
}

#[test]
fn test_001_returns_dbsize() {
    use std::net::{IpAddr, Ipv4Addr};

    let config = Config::new(String::from("./src/redis.conf"));
    let db = Database::new("filename".to_string());
    let database = Arc::new(RwLock::new(db));
    let conf = Arc::new(RwLock::new(config));
    let operation = RespType::RArray(vec![RespType::RBulkString("dbsize".to_string())]);
    let (tx, _sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    handle_command(operation, &tx, addrs, &database, &conf);
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
}

#[test]
fn test_003_cleans_db_items() {
    use std::net::{IpAddr, Ipv4Addr};

    let db = Database::new("filename".to_string());
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
}

#[test]
fn test_004_deletes_a_key_from_db() {
    use std::net::{IpAddr, Ipv4Addr};

    let db = Database::new("filename".to_string());
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
}

#[test]
fn test_005_check_if_key_exists_throws_zero() {
    use std::net::{IpAddr, Ipv4Addr};

    let db = Database::new("filename".to_string());
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
}

#[test]
fn test_006_check_if_key_exists_throws_one() {
    use std::net::{IpAddr, Ipv4Addr};

    let db = Database::new("filename".to_string());
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
}
