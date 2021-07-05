use std::{
    net::{SocketAddr, TcpStream},
    sync::mpsc::Sender,
};

use crate::services::utils::resp_type::RespType;

// use super::config::Config;

pub enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
pub enum WorkerMessage {
    Log(String),
    Verb(String),
    NewOperation(RespType, SocketAddr),
    MonitorOp(String, TcpStream),
    Stop(bool),
    Subscribe(String, SocketAddr, Sender<String>),
    Unsubscribe(String, SocketAddr),
    UnsubscribeAll(SocketAddr),
    Publish(String, Sender<usize>, String), // Request(TcpStream, Sender<WorkerMessage>, Arc<RwLock<Database>>, Arc<RwLock<Config>>, Sender<bool>)
}
