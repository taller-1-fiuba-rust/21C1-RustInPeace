use crate::domain::entities::config::Config;
use crate::domain::entities::message::WorkerMessage;
use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, RwLock};
use std::{net::SocketAddr, sync::mpsc::Sender};

pub fn monitor(tx: &Sender<WorkerMessage>, addrs: &SocketAddr) {
    tx.send(WorkerMessage::MonitorOp(addrs.to_string()))
        .unwrap();
}

pub fn info(cmd: &[RespType]) -> RespType {
    let mut option = "default".to_string();
    if cmd.len() == 2 {
        if let RespType::RBulkString(comando) = &cmd[1] {
            option = comando.to_string();
        }
    }

    match option.as_str() {
        "server" => {
            RespType::RBulkString("# Server\r\nredis_version:6.2.3\r\nredis_git_sha1:00000000\r\nredis_git_dirty:0\r\nredis_build_id:ea3be5cbc55dfd19\r\n".to_string())
        }
        "clients" => {
            RespType::RBulkString("# Clients\r\nconnected_clients:2\r\ncluster_connections:0\r\nmaxclients:10000\r\n".to_string())
        }
        "persistence" => RespType::RNullArray(),
        // "stats" => {}
        // "replication" => {}
        // "cpu" => {}
        // "commandstats" => {}
        // "cluster" => {}
        // "modules" => {}
        // "keyspace" => {}
        // "errorstats" => {}
        // "all" => {}
        // "default" => {}
        // "everything" => {}
        _ => RespType::RNullArray(),
    }
}

pub fn dbsize(database: &Arc<RwLock<Database>>) -> RespType {
    RespType::RInteger(database.read().unwrap().get_size())
}

pub fn config_get(config: &Arc<RwLock<Config>>, field: &RespType) -> Result<String, Error> {
    if let RespType::RBulkString(field_name) = field {
        if let Ok(read_guard) = config.read() {
            let conf = read_guard;
            let value = conf.get_attribute(String::from(field_name))?;
            return Ok(value);
        }
        Err(Error::new(ErrorKind::InvalidInput, "Field name missing"))
    } else {
        Err(Error::new(ErrorKind::InvalidInput, "Invalid request"))
    }
}

pub fn config_set(
    config: &Arc<RwLock<Config>>,
    field: &RespType,
    value: &RespType,
) -> Result<String, Error> {
    if let RespType::RBulkString(field_name) = field {
        if let RespType::RBulkString(value) = value {
            if let Ok(write_guard) = config.write() {
                let mut conf = write_guard;
                let _value = conf.set_attribute(String::from(field_name), String::from(value))?;
                return Ok(String::from("ok"));
            }
        }
        Err(Error::new(ErrorKind::InvalidInput, "Field name missing"))
    } else {
        Err(Error::new(ErrorKind::InvalidInput, "Invalid request"))
    }
}

// #[test]
// fn test_config_get_verbose() {
//     let _parsed_command = RespType::RBulkString(String::from("verbose"));
// }
