use super::utils::resp_type::RespType;
use crate::{
    domain::entities::{config::Config, message::WorkerMessage},
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
    config: &Arc<RwLock<Config>>,
) {
    if let RespType::RArray(array) = operation {
        if let RespType::RBulkString(part_command) = &array[0] {
            match part_command.as_str() {
                "monitor" => {
                    tx.send(WorkerMessage::MonitorOp(addrs.to_string()))
                        .unwrap();
                    // match self.operations.get(&addrs.to_string()) {
                    // Some(operations) => {
                    //     let last_ops = operations.get_operations();
                    //     command_server::monitor(last_ops);
                    // }
                    // None => println!("Client doesnt exist"),
                }
                "info" => println!("completar.."),
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
                }
                _ => {}
            }
        }
    }
}
