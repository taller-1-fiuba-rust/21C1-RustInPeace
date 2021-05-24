//#[derive(Debug)]
use super::worker_service::ThreadPool;
use crate::entities::server::Server;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

pub fn init(server: Server) {
    let port: &String = server.get_port();
    let dir: &String = server.get_dir();
    let threadpool_size: &usize = server.get_threadpool_size();
    let pool = ThreadPool::new(*threadpool_size);
    match TcpListener::bind(format!("{}:{}", dir, port)) {
        Ok(listener) => {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        match stream.peer_addr() {
                            Ok(addrs) => {
                                println!("New stream: {}", addrs);
                            }
                            Err(_) => {
                                println!("Could't get client's address");
                            }
                        }
                        //if timeout != 0 {
                        //stream.set_read_timeout(Some(Duration::from_millis(timeout)));//handle err
                        //}
                        pool.spawn(move || {
                            handle_connection(stream);
                        });
                    }
                    Err(_) => {
                        println!("Couldn't get stream");
                    }
                }
            }
        }
        Err(_) => {
            println!("Listener couldn't be created");
        }
    }

    println!("Shutting down.");
}

//lo pongo como _stream porque todavia no implementamos esto
fn handle_connection(_stream: TcpStream) {
    std::thread::sleep(Duration::from_secs(2));
    println!("handle_connection says Hi!");
}
