use super::utils::resp_type::RespType;
use crate::domain::implementations::database::Database;
use crate::services::commands::{command_pubsub, command_set};
use crate::{
    domain::entities::{config::Config, message::WorkerMessage},
    services::commands::command_key,
    services::commands::command_list,
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

/// Delega el comando ingresado por el cliente al servicio de comandos que corresponda.
///
/// Los comandos posibles son:
/// * monitor
/// * info
/// * config
/// * dbsize
/// * flushdb
/// * copy
/// * del
/// * exists
/// * persist
/// * rename
/// * expire
/// * expireat
/// * sort
/// * keys
/// * touch
/// * type
/// * append
/// * decrby
/// * incrby
/// * get
/// * getdel
/// * getset
/// * copy
/// * strlen
/// * get
/// * mget
/// * mset
/// * subscribe
/// * unsubscribe
/// * punsubscribe
/// * pubsub
/// * publish
/// * ttl
/// * command
/// * lpush
/// * lpushx
/// * llen
/// * lrange
/// * lindex
/// * lpop
/// * sadd
/// * scard
/// * sismember
/// * smembers
/// * srem
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
                "monitor" => command_server::monitor(&tx, addrs),
                "info" => return Some(command_server::info(&array, tx)),
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
                "subscribe" => return Some(command_pubsub::subscribe(&array, tx, addrs, stream)),
                "unsubscribe" => return Some(command_pubsub::unsubscribe(&array, tx, addrs)),
                "punsubscribe" => {
                    //no se pide implementar esta funcion pero la agrego hardcodeada -por ahora- porque el cliente Redis la llama despues de un subscribe
                    return Some(RespType::RArray(vec![
                        RespType::RBulkString(String::from("unsubscribe")),
                        RespType::RBulkString(String::from("foo")),
                        RespType::RInteger(0),
                    ]));
                }
                "pubsub" => return Some(command_pubsub::pubsub(&array, tx)),
                "publish" => return Some(command_pubsub::publish(&array, tx)),
                "ttl" => return Some(command_key::get_ttl(&array, database)),
                "command" => {
                    return Some(RespType::RArray(vec![
                        RespType::RBulkString(String::from("append")),
                        RespType::RBulkString(String::from("pubsub")),
                    ]))
                }
                "lpush" => return Some(command_list::push(&array, database, true)),
                "lindex" => return Some(command_list::get_index(&array, database)),
                "llen" => return Some(command_list::llen(&array, database)),
                "lpop" => return Some(command_list::lpop(&array, database)),
                "sadd" => return Some(command_set::add(&array, database)),
                "lpushx" => return Some(command_list::lpushx(&array, database)),
                "lrange" => return Some(command_list::lrange(&array, database)),
                "lset" => return Some(command_list::lset(&array, database)),
                "rpop" => return Some(command_list::rpop(&array, database)),
                "rpushx" => return Some(command_list::rpushx(&array, database)),
                "lrem" => return Some(command_list::lrem(&array, database)),
                "scard" => return Some(command_set::scard(&array, database)),
                "sismember" => return Some(command_set::sismember(&array, database)),
                "smembers" => return Some(command_set::smembers(&array, database)),
                "srem" => return Some(command_set::srem(&array, database)),
                "rpush" => return Some(command_list::push(&array, database, false)),
                _ => {}
            }
        }
    }
    None
}
