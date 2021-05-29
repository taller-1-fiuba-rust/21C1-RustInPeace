//#[derive(Debug)]
use super::protocol_parser_service::{parse_request, parse_response};
use super::worker_service::ThreadPool;
use crate::entities::resp_types::RespTypes;
use crate::entities::server::Server;
use std::io::{BufRead, BufReader};
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
fn handle_connection(stream: TcpStream) {
    std::thread::sleep(Duration::from_secs(2));
    println!("handle_connection says Hi!");
    //llega un mensaje, lo leemos y lo mandamos al parser service
    let reader = BufReader::new(stream);
    let lines = reader.lines();
    // iteramos las lineas que recibimos de nuestro cliente
    for line in lines {
        println!("Recibido: {:?}", line);
        //println!("Recibido as bytes: {:?}", line.unwrap().as_bytes());
        let bytes = line.unwrap();
        let message = bytes.as_bytes();
        println!("recibido as bytes: {:?}", message);
        match parse_request(message) {
            Ok(parsed_request) => {
                println!("Parsed request: {:?}", parsed_request);
                // de aca pasa a un servicio que delegue segun la request que sea
                // ese servicio va a devolver una response
                // simulo una response:
                let response = parse_response(RespTypes::RInteger(5));
                println!("Parsed response: {}", response);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}
