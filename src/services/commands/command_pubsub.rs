//! Servicio que implementa todos los comandos Pub/Sub

use std::{
    net::SocketAddr,
    sync::mpsc::{self, Sender},
};

use crate::{domain::entities::message::WorkerMessage, services::utils::resp_type::RespType};

/// Suscribe un cliente al canal indicado.
///
/// Una vez que el cliente se suscribe a un canal, no puede ejecutar ningún otro comando.
/// `Subscribe` es una función bloqueante, sólo recibe mensajes que hayan sido publicados al canal.
/// Devuelve el nombre del canal y la cantidad de clientes suscritos al canal.
pub fn subscribe(cmd: &[RespType], tx: &Sender<WorkerMessage>, addrs: SocketAddr) -> RespType {
    let (messages_sender, messages_receiver) = mpsc::channel();

    for channel in &cmd[1..] {
        if let RespType::RBulkString(channel) = channel {
            tx.send(WorkerMessage::Subscribe(
                channel.to_string(),
                addrs,
                messages_sender.clone(),
            ))
            .expect("Could not send Subscribe message");

            if let Ok(n_channels) = messages_receiver.recv() {
                return RespType::RArray(vec![
                    RespType::RBulkString(String::from("subscribe")),
                    RespType::RBulkString(channel.to_string()),
                    RespType::RInteger(n_channels),
                ]);
            }
        }
    }
    RespType::RArray(vec![])
}

/// Desuscribe un cliente del canal indicado.
///
/// Si no se espefica un canal, se lo desuscribe de todos a los que se haya suscrito.
/// Devuelve el nombre del canal y la cantidad de suscriptores actualizada.
pub fn unsubscribe(cmd: &[RespType], tx: &Sender<WorkerMessage>, addrs: SocketAddr) -> RespType {
    let (messages_sender, messages_receiver) = mpsc::channel();
    if cmd.len() > 1 {
        for channel in &cmd[1..] {
            if let RespType::RBulkString(channel) = channel {
                tx.send(WorkerMessage::Unsubscribe(
                    channel.to_string(),
                    addrs,
                    messages_sender.clone(),
                ))
                .expect("Could not send Unsubscribe message");

                if let Ok(n_channels) = messages_receiver.recv() {
                    return RespType::RArray(vec![
                        RespType::RBulkString(String::from("unsubscribe")),
                        RespType::RBulkString(channel.to_string()),
                        RespType::RInteger(n_channels),
                    ]);
                }
            }
        }
    } else {
        tx.send(WorkerMessage::UnsubscribeAll(addrs, messages_sender))
            .expect("Could not send UnsubscribeAll message");
        if let Ok(n_channels) = messages_receiver.recv() {
            return RespType::RArray(vec![
                RespType::RBulkString(String::from("unsubscribe")),
                RespType::RBulkString("bar".to_string()),
                RespType::RInteger(n_channels),
            ]);
        }
    }
    RespType::RArray(vec![])
}

/// Publica un mensaje en el canal pedido.
///
/// A cada cliente suscrito al canal especificado se le envía, además del mensaje, el canal por el cual llega.
/// Este comando devuelve la cantidad de clientes que recibieron el mensaje.
pub fn publish(cmd: &[RespType], tx: &Sender<WorkerMessage>) -> RespType {
    if let RespType::RBulkString(channel) = &cmd[1] {
        if let RespType::RBulkString(message) = &cmd[2] {
            let (response_sender, response_receiver) = mpsc::channel();
            tx.send(WorkerMessage::Publish(
                channel.to_string(),
                response_sender,
                message.to_string(),
            ))
            .expect("Could not send publish message");

            if let Ok(res) = response_receiver.recv() {
                return RespType::RInteger(res);
            }
        }
    }
    RespType::RInteger(0)
}

/// Lista canales activos o el numero de suscriptores de los canales especificados.
///
/// Si el comando es seguido por "channels" se listan todos los canales activos.
/// Si el comando es seguido por "numsub" se listan los canales especificados y el numero de suscriptores.
pub fn pubsub(cmd: &[RespType], tx: &Sender<WorkerMessage>) -> RespType {
    if let RespType::RBulkString(command) = &cmd[1] {
        match command.as_str() {
            "channels" => {
                return pubsub_channels(cmd, tx);
            }
            "numsub" => {
                return pubsub_numsub(cmd, tx);
            }
            _ => {}
        }
    }
    RespType::RArray(vec![])
}

/// Lista canales activos.
///
/// Canales activos son aquellos que tengan al menos un suscriptor.
/// Si se especifica un patrón, se listan los canales cuyo nombre cumplan el patrón,
/// sino se listan todos.
fn pubsub_channels(cmd: &[RespType], tx: &Sender<WorkerMessage>) -> RespType {
    let (response_sender, response_receiver) = mpsc::channel();
    if cmd.len() >= 3 {
        if let RespType::RBulkString(pattern) = &cmd[2] {
            tx.send(WorkerMessage::Channels(
                response_sender,
                Some(pattern.to_string()),
            ))
            .expect("Could not send Channels message");

            if let Ok(res) = response_receiver.recv() {
                return RespType::RArray(res);
            }
        }
    } else {
        tx.send(WorkerMessage::Channels(response_sender, None))
            .expect("Could not send Channels message");

        if let Ok(res) = response_receiver.recv() {
            return RespType::RArray(res);
        }
    }
    RespType::RArray(vec![])
}

/// Devuelve el numero de suscriptores por cada canal.
///
/// Retorna una lista de canales y su cantidad de suscriptores en la forma (canal, cantidad).
/// El orden de la lista es el mismo que en los parametros del comando [chequear esto]
fn pubsub_numsub(cmd: &[RespType], tx: &Sender<WorkerMessage>) -> RespType {
    let (messages_sender, messages_receiver) = mpsc::channel();
    let mut channels = Vec::new();
    for channel in &cmd[2..] {
        if let RespType::RBulkString(channel) = channel {
            channels.push(channel.to_string());
        }
    }
    tx.send(WorkerMessage::Numsub(channels, messages_sender))
        .expect("Could not send Numsub message");

    if let Ok(res) = messages_receiver.recv() {
        return RespType::RArray(res);
    }
    RespType::RArray(vec![])
}
