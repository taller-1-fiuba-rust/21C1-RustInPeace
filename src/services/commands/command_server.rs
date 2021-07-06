use crate::domain::entities::config::Config;
use crate::domain::entities::message::WorkerMessage;
use crate::domain::implementations::database::Database;
use crate::services::utils::resp_type::RespType;
use std::net::TcpStream;
use std::sync::{Arc, RwLock};
use std::{sync::mpsc::Sender};

pub fn monitor(tx: &Sender<WorkerMessage>, stream: TcpStream) {
    tx.send(WorkerMessage::MonitorOp(stream)).unwrap();
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

/// Recibe la base de datos database dentro de un RwLock
/// Devuelve la cantidad de claves almacenadas en la base
pub fn dbsize(database: &Arc<RwLock<Database>>) -> RespType {
    RespType::RInteger(database.read().unwrap().get_size())
}

//hay que hacerlo con las opciones sync/async??
/// Recibe un comando cmd de tipo &[RespType] y la base de datos database dentro de un RwLock
/// Elimina todas las claves y valores almacenados en la base de datos
/// Devuelve el mensaje "Erased database"
pub fn flushdb(database: &Arc<RwLock<Database>>) -> RespType {
    let mut new_database = database.write().unwrap();
    new_database.clean_items();
    RespType::RBulkString("Erased database".to_string())
}

/// Recibe la configuración config y un campo field de tipo &RespType
/// Busca el valor del atributo de nombre field en la configuración
/// En caso de encontrarlo, lo devuelve como un Simple String, sino devuelve Error.
pub fn config_get(config: &Arc<RwLock<Config>>, field: &RespType) -> RespType {
    if let RespType::RBulkString(field_name) = field {
        if let Ok(conf) = config.read() {
            match conf.get_attribute(String::from(field_name)) {
                Ok(value) => {
                    return RespType::RArray(vec![RespType::RSimpleString(value)]);
                } //Hay que ajustar esta funcion
                Err(_e) => {}
            }
        }
        RespType::RError(String::from("Field name missing"))
    } else {
        RespType::RError(String::from("Invalid request"))
    }
}

/// Recibe la configuración config, un campo field de tipo &RespType y un valor value de tipo &RespType
/// Setea el campo field de la configuración con el valor value
/// En caso de exito devuelve un Simple String "ok", sino devuelve Error
pub fn config_set(config: &Arc<RwLock<Config>>, field: &RespType, value: &RespType) -> RespType {
    if let RespType::RBulkString(field_name) = field {
        if let RespType::RBulkString(value) = value {
            if let Ok(mut conf) = config.write() {
                match conf.set_attribute(String::from(field_name), String::from(value)) {
                    Ok(_) => {
                        return RespType::RSimpleString(String::from("ok"));
                    }
                    Err(e) => {
                        return RespType::RError(e.to_string());
                    }
                }
            }
        }
        RespType::RError(String::from("Field name missing"))
    } else {
        RespType::RError(String::from("Invalid request"))
    }
}
