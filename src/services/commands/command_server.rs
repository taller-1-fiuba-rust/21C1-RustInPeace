use crate::services::utils::resp_type::RespType;
use crate::domain::entities::message::WorkerMessage;
use std::{net::SocketAddr, sync::mpsc::Sender};
use crate::domain::implementations::database::Database;
use std::sync::{Arc,RwLock};

pub fn monitor(tx: &Sender<WorkerMessage>, addrs: &SocketAddr) {
    tx.send(WorkerMessage::MonitorOp(addrs.to_string()))
    .unwrap();
}

pub fn info(cmd: &Vec<RespType>) -> RespType {
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