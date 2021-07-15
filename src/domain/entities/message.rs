use super::client::Client;

use crate::services::utils::resp_type::RespType;
use std::{
    net::{SocketAddr, TcpStream},
    sync::mpsc::Sender,
};

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
    SetMonitor(SocketAddr),
    AddClient(Client),
    CloseClient(SocketAddr),
    Stop(bool),
    Subscribe(String, SocketAddr, Sender<usize>, TcpStream),
    Unsubscribe(String, SocketAddr, Sender<usize>),
    UnsubscribeAll(SocketAddr, Sender<usize>),
    Publish(String, Sender<usize>, String),
    Channels(Sender<Vec<RespType>>, Option<String>),
    Numsub(Vec<String>, Sender<Vec<RespType>>),
}
