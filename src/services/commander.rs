use super::commands::command_pubsub;
use super::utils::resp_type::RespType;
use crate::domain::entities::key_value_item::ValueTimeItem; //, ValueType};
use crate::domain::implementations::database::Database;
use crate::services::commands::command_list;
use crate::{
    domain::entities::{config::Config, message::WorkerMessage},
    services::commands::command_key,
    services::commands::command_server,
    services::commands::command_string,
};
#[allow(unused)]
use std::fs::File;
use std::net::TcpStream;
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
    stream: TcpStream,
) -> Option<RespType> {
    if let RespType::RArray(array) = operation {
        if let RespType::RBulkString(actual_command) = &array[0] {
            match actual_command.as_str() {
                "monitor" => command_server::monitor(&tx, &addrs, stream),
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
                "dbsize" => return Some(command_server::dbsize(&database)),
                "flushdb" => return Some(command_server::flushdb(database)),
                "copy" => return Some(command_key::copy(&array, database)),
                "del" => return Some(command_key::del(&array, database)),
                "exists" => return Some(command_key::exists(&array, database)),
                "persist" => return Some(command_key::persist(&array, database)),
                "rename" => return Some(command_key::rename(&array, database)),
                "expire" => return Some(command_key::expire(&array, database)),
                "expireat" => return Some(command_key::expireat(&array, database)),
                "sort" => return Some(command_key::sort(&array, database)),
                "keys" => return Some(command_key::keys(&array, database)),
                "touch" => return Some(command_key::touch(&array, database)),
                "type" => return Some(command_key::get_type(&array, database)),
                "append" => return Some(command_string::append(&array, database)),
                "decrby" => return Some(command_string::decrby(&array, database)),
                "get" => return Some(command_string::get(&array, database)),
                "getdel" => return Some(command_string::getdel(&array, database)),
                "getset" => return Some(command_string::getset(&array, database)),
                "incrby" => return Some(command_string::incrby(&array, database)),
                "strlen" => return Some(command_string::strlen(&array, database)),
                "mget" => return Some(command_string::mget(&array, database)),
                "mset" => return Some(command_string::mset(&array, database)),
                "set" => return Some(command_string::set(&array, database)),
                "subscribe" => command_pubsub::subscribe(&array, tx, addrs, stream),
                "unsubscribe" => {
                    command_pubsub::unsubscribe(&array, tx, addrs);
                    return Some(RespType::RNullBulkString());
                }
                "publish" => return Some(command_pubsub::publish(&array, tx)),
                "ttl" => return Some(command_key::get_ttl(&array, database)),
                "lindex" => return Some(command_list::get_index(&array, database)),
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

pub fn get_database_size(database: &Arc<RwLock<Database>>) -> usize {
    if let Ok(write_guard) = database.read() {
        write_guard.get_size()
    } else {
        0
    }
}
