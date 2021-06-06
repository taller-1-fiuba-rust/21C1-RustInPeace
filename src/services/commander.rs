use super::utils::resp_type::RespType;
use crate::domain::entities::message::WorkerMessage;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{self, channel};
use std::{net::SocketAddr, sync::mpsc::Sender};
use crate::domain::entities::server::Server;
use crate::domain::implementations::database::Database;
use std::sync::{Arc,RwLock};
use crate::services::commands::command_server;

pub fn handle_command(operation: RespType, tx: &Sender<WorkerMessage>, addrs: SocketAddr, database: &Arc<RwLock<Database>>) {
    if let RespType::RArray(array) = operation {
        if let RespType::RBulkString(actual_command) = &array[0] {
            match actual_command.as_str() {
                "monitor" => {
                    command_server::monitor(&tx,&addrs);
                    // match self.operations.get(&addrs.to_string()) {
                    // Some(operations) => {
                    //     let last_ops = operations.get_operations();
                    //     command_server::monitor(last_ops);
                    // }
                    // None => println!("Client doesnt exist"),
                }
                "info" => {
                    let infor_requiered = command_server::info(&array);
                        println!("{:?}", infor_requiered);
                }

                "dbsize" => {
                    // let db_size = database.read().unwrap().get_size(); //database.read().unwrap().get_size();
                    // println!("database size: {:?}" , db_size);
                    let db_size = command_server::dbsize(&database);
                    println!("database size: {:?}" , db_size);
                
                }

                "flushdb" => {


                }
                _ => {}
            }
        }
    }
}

#[test]
fn test_001() {
    let db = Database::new("filename".to_string());
    let mut database = Arc::new(RwLock::new(db));
    let operation = RespType::RArray(vec![RespType::RBulkString("dbsize".to_string())]);
    let (tx,_sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),8080);
    handle_command(operation, &tx, addrs, &database)

}

#[test]
fn test_002() {
    let db = Database::new("filename".to_string());
    let mut database = Arc::new(RwLock::new(db));
    let operation = RespType::RArray(vec![RespType::RBulkString("info".to_string()), RespType::RBulkString("server".to_string())]);
    let (tx,_sx) = std::sync::mpsc::channel();
    let addrs = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),8080);
    handle_command(operation, &tx, addrs, &database)

}