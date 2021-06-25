use std::{net::SocketAddr, sync::mpsc::{self, Sender}};

use crate::{domain::entities::message::WorkerMessage, services::utils::resp_type::RespType};

/// Recibe un comando cmd de tipo &[RespType]
/// Extrae el nombre del channel del comando, suscribe al cliente a dicho canal.
pub fn subscribe(cmd: &[RespType], tx: &Sender<WorkerMessage>, addrs: SocketAddr) {
    if let RespType::RBulkString(channel) = &cmd[1] {
        //creo channel para comunicar 
        let (messages_sender, messages_receiver) = mpsc::channel();
        tx.send(WorkerMessage::Subscribe(channel.to_string(), addrs, messages_sender)).unwrap();
        println!("Reading messages...");
        //algo para QUIT
        for message in messages_receiver {
            println!("{}", message);
        }
    }
}