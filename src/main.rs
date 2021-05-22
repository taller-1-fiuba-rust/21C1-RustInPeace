mod services;
mod entities;

use services::worker_service::ThreadPool;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::time::{Instant,Duration};
use std::io::BufReader;
use std::io::{ErrorKind, Read, Write};

fn main() {
    let dir: String = "127.0.0.1".to_string();
    //hardcodeo configuracion
    let verbose: usize = "1".parse().unwrap(); //to handle
    let port: String = "8080".to_string();
    let timeout: u64 = "10".parse().unwrap(); //to handle
    let dbfilename = "custom.rbd";
    let logfile = "log.out";

    let listener = TcpListener::bind(format!("{}:{}", dir, port)).unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("new stream: {}", stream.peer_addr().unwrap());
        if timeout != 0 {
            //stream.set_read_timeout(Some(Duration::from_millis(timeout)));//handle err
        }
        pool.spawn(move|| {
            handle_connection(stream, timeout);
            
        });
    }
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream, timeout: u64) {
    println!("handle_connection says Hi!");

}

//#[test]
