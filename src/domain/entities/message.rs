use std::net::SocketAddr;

use crate::services::utils::resp_type::RespType;

pub enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub enum WorkerMessage {
    Log(String),
    NewOperation(RespType, SocketAddr),
    MonitorOp(String),
}
