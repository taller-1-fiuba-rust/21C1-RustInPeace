use super::utils::resp_type::RespType;
use crate::domain::entities::message::WorkerMessage;
use std::{net::SocketAddr, sync::mpsc::Sender};

pub fn handle_command(operation: RespType, tx: &Sender<WorkerMessage>, addrs: SocketAddr) {
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
                _ => {}
            }
        }
    }
}
