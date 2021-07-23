//! Tipos de mensajes para enviar entre canales.

use super::client::Client;
use crate::services::utils::resp_type::RespType;
use std::{
    net::{SocketAddr, TcpStream},
    sync::mpsc::Sender,
};

/// Tipo de mensaje para comunicar al threadpool con sus workers.
/// El threadpool puede enviar dos tipos de mensajes:
/// * NewJob: directiva para que algún worker atienda un cliente nuevo.
/// * Terminate: directiva para que el worker deje de atender clientes.
pub enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Tipo de mensaje para comunicar al cliente con el servidor.
/// El cliente puede enviar los siguientes mensajes al servidor:
/// * Log: envia un mensaje que debe escribirse en el archivo de log.
/// * Verb: envia un mensaje para que imprima el servidor.
/// * NewOperation: envia el último comando ejecutado.
/// * SetMonitor: registra que el cliente ejecutó el comando `monitor`.
/// * InfoServer: pide información del servidor.
/// * InfoClients: pide información de los clientes conectados al servidor.
/// * InfoStats: pide estadísticas sobre el uso del servidor.
/// * AddClient: registra al nuevo cliente conectado.
/// * CloseClient: elimina un cliente del registro de clientes conectados.
/// * Subscribe: suscribe un cliente a un canal.
/// * Unsubscribe: desuscribe un cliente de un canal.
/// * UnsubscribeAll: desuscribe un cliente de todos los canales.
/// * Publish: publica un mensaje en un canal.
/// * Channels: pide los nombres de los canales que cumplen con cierto patrón.
/// * Numsub: pide la cantidad de suscriptores por canal.
#[derive(Debug)]
pub enum WorkerMessage {
    Log(String),
    Verb(String),
    NewOperation(RespType, SocketAddr),
    SetMonitor(SocketAddr),
    InfoServer(Sender<String>),
    InfoClients(Sender<String>),
    InfoStats(Sender<String>),
    AddClient(Client),
    CloseClient(SocketAddr),
    Subscribe(String, SocketAddr, Sender<usize>, TcpStream),
    Unsubscribe(String, SocketAddr, Sender<usize>),
    UnsubscribeAll(SocketAddr, Sender<usize>),
    Publish(String, Sender<usize>, String),
    Channels(Sender<Vec<RespType>>, Option<String>),
    Numsub(Vec<String>, Sender<Vec<RespType>>),
}
