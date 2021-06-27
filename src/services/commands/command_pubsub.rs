use std::{net::SocketAddr, sync::mpsc::{self, Sender}, thread};

use crate::{domain::entities::message::WorkerMessage, services::utils::resp_type::RespType};

/// Recibe un comando cmd de tipo &[RespType]
/// Extrae el nombre del channel del comando, suscribe al cliente a dicho canal.
/// Para suscribirlo, crea un canal mpsc y envia un WorkerMessage Subscribe mediante el sender tx
/// En el WorkerMessage se envian el canal al que se quiere suscribir, la direccion del cliente que se suscribe y 
/// un sender para que desde otro hilo de ejecución se puedan enviar mensajes al nuevo canal creado
/// En un loop for se reciben todos los mensajes que lleguen al canal al que fue suscrito el cliente
pub fn subscribe(cmd: &[RespType], tx: &Sender<WorkerMessage>, addrs: SocketAddr) {
    // if let RespType::RBulkString(channel) = &cmd[1] {
        //creo channel para comunicar 
        let (messages_sender, messages_receiver) = mpsc::channel();

        for channel in &cmd[1..] {
            println!("{:?}", &cmd[1..]);
            if let RespType::RBulkString(channel) = channel {
                tx.send(WorkerMessage::Subscribe(channel.to_string(), addrs, messages_sender.clone())).unwrap();
            }
        }
        
        println!("Reading messages...");
        //algo para QUIT

        //en thread aparte?
        //se tiene que cerrar con CTRL-C
        // thread::spawn(|| {
            println!("tamo aca esperando");
            for message in messages_receiver {
                println!("llego un mensaje {}", message);
                if message == "QUIT" {
                    break;
                }
                println!("{}", message);
            }
            println!("llego aca");
            // drop(messages_sender);
        // });
        println!("bye");
    
        // h.join().unwrap();
    // }
}

//Unsubscribes the client from the given channels, or from all of them if none is given.
//When no channels are specified, the client is unsubscribed from all the previously subscribed channels. 
//In this case, a message for every unsubscribed channel will be sent to the client.
pub fn unsubscribe(cmd: &[RespType], tx: &Sender<WorkerMessage>, addrs: SocketAddr) {
    println!("unsubs...");
    if cmd.len() > 1 {
        for channel in &cmd[1..] {
            println!("{:?}", &cmd[1..]);
            if let RespType::RBulkString(channel) = channel {
                println!("channel to unsub {}", channel);
                tx.send(WorkerMessage::Unsubscribe(channel.to_string(), addrs)).unwrap();
            }
        }
    } else {
        println!("unsub all");
        tx.send(WorkerMessage::UnsubscribeAll(addrs)).unwrap();
    }   
}

///Posts a message to the given channel.
///In a Redis Cluster clients can publish to every node. The cluster makes sure that published messages are forwarded as needed, 
///so clients can subscribe to any channel by connecting to any one of the nodes.
///Return value -> Integer reply: the number of clients that received the message.
pub fn publish(cmd: &[RespType], tx: &Sender<WorkerMessage>, addrs: SocketAddr) -> RespType {
    if let RespType::RBulkString(channel) = &cmd[1] {
        if let RespType::RBulkString(message) = &cmd[2] {
            let (response_sender, response_receiver) = mpsc::channel();
            tx.send(WorkerMessage::Publish(channel.to_string(), response_sender, message.to_string())).unwrap();

            let res = response_receiver.recv().unwrap();
            println!("res: {}", res);
            return RespType::RInteger(res);
        }
    }
    println!("oops");
    RespType::RInteger(0)
}