use std::{net::{SocketAddr, TcpStream}, sync::{Arc, RwLock, mpsc::Sender}};

use crate::{domain::implementations::database::Database, services::utils::resp_type::RespType};

use super::config::Config;

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
    MonitorOp(String),
    Stop(bool),
    // Request(TcpStream, Sender<WorkerMessage>, Arc<RwLock<Database>>, Arc<RwLock<Config>>, Sender<bool>)
}
