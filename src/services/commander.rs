//! Servicio que delega el comando ingresado según su tipo.
//! Los tipos pueden ser: list, key, server, string, pubsub y set.

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
use std::error::Error;
#[allow(unused)]
use std::fs::File;
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
///
/// Devuelve un Option de tipo RespType con la respuesta que se le devolverá al cliente.
///
/// # Ejemplo
/// ```ignore
/// let listener = TcpListener::bind(format!("{}:{}", dir, port)).unwrap();
/// for stream in listener.incoming() {
///     let stream = stream.unwrap();
///     let mut buf = [0u8; 512];
///     let msg_len = stream.read(&mut buf).unwrap();
///     let msg = &buf[..msg_len];
///     let parsed_msg = parser_service::parse_request(msg).unwrap();
///     commander::parser_service(parsed_msg, &server_sender, stream.peer_addrs().unwrap(), database, config, stream);
/// }
/// ```
pub fn handle_command(
    operation: RespType,
    tx: &Sender<WorkerMessage>,
    addrs: SocketAddr,
    database: &Arc<RwLock<Database>>,
    config: &Arc<RwLock<Config>>,
    subscribed: bool,
) -> Result<RespType, Box<dyn Error>> {
    if let RespType::RArray(array) = operation {
        if let RespType::RBulkString(actual_command) = &array[0] {
            if subscribed && !get_pubsub_commands().contains(actual_command) {
                return Err(
                    "In subscribed state. Commands allowed: [subscribe, unsubscribe]".into(),
                );
            }
            match actual_command.as_str() {
                "monitor" => command_server::monitor(&tx, addrs),
                "info" => return Ok(command_server::info(&array, tx)),
                "config" => {
                    if let RespType::RBulkString(instruction) = &array[1] {
                        match instruction.as_str() {
                            "get" => {
                                return Ok(command_server::config_get(config, &array[1..]));
                            }
                            "set" => {
                                return Ok(command_server::config_set(config, &array[1..]));
                            }
                            _ => {}
                        }
                    }
                }
                "dbsize" => return Ok(command_server::dbsize(&database)),
                "flushdb" => return Ok(command_server::flushdb(database)),
                "copy" => return Ok(command_key::copy(&array, database)),
                "del" => return Ok(command_key::del(&array, database)),
                "exists" => return Ok(command_key::exists(&array, database)),
                "persist" => return Ok(command_key::persist(&array, database)),
                "rename" => return Ok(command_key::rename(&array, database)),
                "expire" => return Ok(command_key::expire(&array, database)),
                "expireat" => return Ok(command_key::expireat(&array, database)),
                "sort" => return Ok(command_key::sort(&array, database)),
                "keys" => return Ok(command_key::keys(&array, database)),
                "touch" => return Ok(command_key::touch(&array, database)),
                "type" => return Ok(command_key::get_type(&array, database)),
                "append" => return Ok(command_string::append(&array, database)),
                "decrby" => return Ok(command_string::decrby(&array, database)),
                "get" => return Ok(command_string::get(&array, database)),
                "getdel" => return Ok(command_string::getdel(&array, database)),
                "getset" => return Ok(command_string::getset(&array, database)),
                "incrby" => return Ok(command_string::incrby(&array, database)),
                "strlen" => return Ok(command_string::strlen(&array, database)),
                "mget" => return Ok(command_string::mget(&array, database)),
                "mset" => return Ok(command_string::mset(&array, database)),
                "set" => return Ok(command_string::set(&array, database)),
                "subscribe" => return Ok(command_pubsub::subscribe(&array, tx, addrs)),
                "unsubscribe" => return Ok(command_pubsub::unsubscribe(&array, tx, addrs)),
                "punsubscribe" => {
                    //no se pide implementar esta funcion pero el cliente Redis la llama despues de un subscribe. Implemento igual que unsubscribe
                    return Ok(command_pubsub::unsubscribe(&array, tx, addrs));
                }
                "pubsub" => return Ok(command_pubsub::pubsub(&array, tx)),
                "publish" => return Ok(command_pubsub::publish(&array, tx)),
                "ttl" => return Ok(command_key::get_ttl(&array, database)),
                "command" => {
                    return Ok(RespType::RArray(
                        get_commands()
                            .iter()
                            .map(|c| RespType::RBulkString(c.to_string()))
                            .collect(),
                    ))
                }
                "lpush" => return Ok(command_list::push(&array, database, true)),
                "lindex" => return Ok(command_list::lindex(&array, database)),
                "llen" => return Ok(command_list::llen(&array, database)),
                "lpop" => return Ok(command_list::lpop(&array, database)),
                "sadd" => return Ok(command_set::add(&array, database)),
                "lpushx" => return Ok(command_list::lpushx(&array, database)),
                "lrange" => return Ok(command_list::lrange(&array, database)),
                "lset" => return Ok(command_list::lset(&array, database)),
                "rpop" => return Ok(command_list::rpop(&array, database)),
                "rpushx" => return Ok(command_list::rpushx(&array, database)),
                "lrem" => return Ok(command_list::lrem(&array, database)),
                "scard" => return Ok(command_set::scard(&array, database)),
                "sismember" => return Ok(command_set::sismember(&array, database)),
                "smembers" => return Ok(command_set::smembers(&array, database)),
                "srem" => return Ok(command_set::srem(&array, database)),
                "rpush" => return Ok(command_list::push(&array, database, false)),
                _ => {}
            }
        }
    }
    Err("Invalid command".into())
}

pub fn get_pubsub_commands() -> Vec<String> {
    vec![
        String::from("subscribe"),
        String::from("unsubscribe"),
        String::from("punsubscribe"),
    ]
}

pub fn get_commands() -> Vec<String> {
    vec![
        String::from("subscribe"),
        String::from("unsubscribe"),
        String::from("punsubscribe"),
        String::from("rpush"),
        String::from("srem"),
        String::from("smembers"),
        String::from("sismember"),
        String::from("scard"),
        String::from("lrem"),
        String::from("rpushx"),
        String::from("rpop"),
        String::from("lset"),
        String::from("lrange"),
        String::from("lpushx"),
        String::from("sadd"),
        String::from("lpop"),
        String::from("llen"),
        String::from("lindex"),
        String::from("lpush"),
        String::from("ttl"),
        String::from("publish"),
        String::from("pubsub"),
        String::from("set"),
        String::from("mset"),
        String::from("mget"),
        String::from("strlen"),
        String::from("incrby"),
        String::from("decrby"),
        String::from("get"),
        String::from("getdel"),
        String::from("getset"),
        String::from("append"),
        String::from("type"),
        String::from("touch"),
        String::from("keys"),
        String::from("sort"),
        String::from("expire"),
        String::from("expireat"),
        String::from("rename"),
        String::from("persist"),
        String::from("exists"),
        String::from("del"),
        String::from("flushdb"),
        String::from("config get"),
        String::from("config set"),
        String::from("dbsize"),
        String::from("monitor"),
        String::from("copy"),
        String::from("info"),
    ]
}
