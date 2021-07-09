use std::{net::{SocketAddr, TcpStream}, sync::mpsc::{self, Sender}};

use crate::{domain::entities::message::WorkerMessage, services::{parser_service, utils::resp_type::RespType}};

/// Suscribe un cliente al canal indicado.
/// Para suscribirlo, crea un canal mpsc y envia un WorkerMessage Subscribe mediante el sender tx
/// En el WorkerMessage se envian el canal al que se quiere suscribir, la direccion del cliente que se suscribe y
/// un sender para que desde otro hilo de ejecución se puedan enviar mensajes al nuevo canal creado
/// En un loop for se reciben todos los mensajes que lleguen al canal al que fue suscrito el cliente
pub fn subscribe(
    cmd: &[RespType],
    tx: &Sender<WorkerMessage>,
    addrs: SocketAddr,
    stream: TcpStream,
) -> RespType {
    //creo channel para comunicar
    let (messages_sender, messages_receiver) = mpsc::channel();

    for channel in &cmd[1..] {
        if let RespType::RBulkString(channel) = channel {
            tx.send(WorkerMessage::Subscribe(
                channel.to_string(),
                addrs,
                messages_sender.clone(),
                stream.try_clone().unwrap()
            ))
            .unwrap();

            if let Ok(n_channels) = messages_receiver.recv() {
                //el integer es la cantidad de canales que esta escuchando
                return RespType::RArray(vec![RespType::RBulkString(String::from("subscribe")), RespType::RBulkString(channel.to_string()), RespType::RInteger(n_channels)]);
            }
        }
    }
    RespType::RArray(vec![])
}

// Desuscribe un cliente del canal indicado
//Unsubscribes the client from the given channels, or from all of them if none is given.
//When no channels are specified, the client is unsubscribed from all the previously subscribed channels.
//In this case, a message for every unsubscribed channel will be sent to the client.
pub fn unsubscribe(cmd: &[RespType], tx: &Sender<WorkerMessage>, addrs: SocketAddr) -> RespType {
    let (messages_sender, messages_receiver) = mpsc::channel();
    if cmd.len() > 1 {
        for channel in &cmd[1..] {
            if let RespType::RBulkString(channel) = channel {
                tx.send(WorkerMessage::Unsubscribe(channel.to_string(), addrs, messages_sender.clone()))
                    .unwrap();

                if let Ok(n_channels) = messages_receiver.recv() {
                    return RespType::RArray(vec![RespType::RBulkString(String::from("unsubscribe")), RespType::RBulkString(channel.to_string()), RespType::RInteger(n_channels)]);
                }
            }
        }
    } else {
        tx.send(WorkerMessage::UnsubscribeAll(addrs, messages_sender)).unwrap();
        if let Ok(n_channels) = messages_receiver.recv() {
            return RespType::RArray(vec![RespType::RBulkString(String::from("unsubscribe")), RespType::RBulkString("all".to_string()), RespType::RInteger(n_channels)]);
        }
    }
    RespType::RArray(vec![])
}

/// Publica un mensaje en todos en el canal pedido
///In a Redis Cluster clients can publish to every node. The cluster makes sure that published messages are forwarded as needed,
///so clients can subscribe to any channel by connecting to any one of the nodes.
///Return value -> Integer reply: the number of clients that received the message.
pub fn publish(cmd: &[RespType], tx: &Sender<WorkerMessage>) -> RespType {
    if let RespType::RBulkString(channel) = &cmd[1] {
        if let RespType::RBulkString(message) = &cmd[2] {
            let (response_sender, response_receiver) = mpsc::channel();
            tx.send(WorkerMessage::Publish(
                channel.to_string(),
                response_sender,
                message.to_string(),
            ))
            .unwrap();

            if let Ok(res) = response_receiver.recv() {
                return RespType::RInteger(res);
            }
        }
    }
    RespType::RInteger(0)
}

///
pub fn pubsub(cmd: &[RespType], tx: &Sender<WorkerMessage>) -> RespType {
    if let RespType::RBulkString(command) = &cmd[1] {
        match command.as_str() {
            "channels" => {
                return pubsub_channels(&cmd, tx);
            }
            "numsub" => {
                return pubsub_numsub(&cmd, tx);
            }
            _ => {}
        }
    }
    RespType::RArray(vec![])
}

fn pubsub_channels(cmd: &[RespType], tx: &Sender<WorkerMessage>) -> RespType {
    let (response_sender, response_receiver) = mpsc::channel();
    if cmd.len() >= 3 {
        if let RespType::RBulkString(pattern) = &cmd[2] {
            tx.send(WorkerMessage::Channels(
                response_sender,
                Some(pattern.to_string())
            ))
            .unwrap();
    
            if let Ok(res) = response_receiver.recv() {
                return RespType::RArray(res);
            }
        }
    } else {
        tx.send(WorkerMessage::Channels(
            response_sender,
            None
        ))
        .unwrap();

        if let Ok(res) = response_receiver.recv() {
            return RespType::RArray(res);
        }
    }
    RespType::RArray(vec![])
}

fn pubsub_numsub(cmd: &[RespType], tx: &Sender<WorkerMessage>) -> RespType {
    let (messages_sender, messages_receiver) = mpsc::channel();
    let mut channels = Vec::new();
    for channel in &cmd[2..] {
        if let RespType::RBulkString(channel) = channel {
            channels.push(channel.to_string());
        }
    }
    tx.send(WorkerMessage::Numsub(
        channels,
        messages_sender.clone(),
    ))
    .unwrap();

    if let Ok(res) = messages_receiver.recv() {
        return RespType::RArray(res);
    }
    RespType::RArray(vec![])
}