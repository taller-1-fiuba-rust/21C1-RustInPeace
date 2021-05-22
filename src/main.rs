mod services;
mod entities;

use services::worker_service::ThreadPool;
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};
use std::str::FromStr;

fn main() {
    let dir: String = "127.0.0.1".to_string();
    //hardcodeo configuracion
    let verbose = "1";
    let port: String = "8080".to_string();
    let timeout: usize = "0".parse().unwrap(); //to handle
    let dbfilename = "custom.rbd";
    let logfile = "log.out";
    let now = Instant::now();

    let listener = TcpListener::bind(format!("{}:{}", dir, port)).unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.spawn(move|| {
            handle_connection(stream);
            
        });
    }
    println!("Shutting down.");
}

fn handle_connection(stream: TcpStream) {
    println!("handle_connection says Hi!");
}