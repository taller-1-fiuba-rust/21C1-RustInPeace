//#[derive(Debug)]
use super::parser_service::{parse_request, parse_response};
use super::utils::resp_type::RespType;
use super::worker_service::ThreadPool;
//use crate::entities::resp_types::RespType;
use crate::entities::server::Server;
use crate::services::commander::Commander;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub fn init(server: Server) {
    let commander = Arc::new(Mutex::new(Commander::new()));
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
                        let shared_commander = Arc::clone(&commander);
                        pool.spawn(move || {
                            handle_connection(stream, shared_commander);
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
fn handle_connection(stream: TcpStream, shared_commander: Arc<Mutex<Commander>>) {
    std::thread::sleep(Duration::from_secs(2));
    println!("handle_connection says Hi!");

    std::thread::sleep(Duration::from_secs(2));
    println!("handle_connection says Hi!");
    let client_addrs = stream.peer_addr().unwrap();
    // leo un mensaje nuevo
    // el mensaje nuevo llega en un arreglo de bytes
    // se lo pasamos a parser
    let reader = BufReader::new(stream);
    let lines = reader.lines();
    // iteramos las lineas que recibimos de nuestro cliente
    for line in lines {
        println!("Recibido: {:?}", line);
        let bytes = line.unwrap();
        let message = bytes.as_bytes();
        println!("recibido as bytes: {:?}", message);
        match parse_request(message) {
            Ok(parsed_request) => {
                println!("Parsed request: {:?}", parsed_request);
                // de aca pasa a un servicio que delegue segun la request que sea
                // ese servicio va a devolver una response
                // simulo una response:
                let response = parse_response(RespType::RInteger(5));
                println!("Parsed response: {}", response);
                // parser devuelve el request en forma de RespType
                // simulo request parseada:
                let elemento_1 = RespType::RBulkString("monitor".to_string());
                let vector_aux = vec![elemento_1];
                let operation = RespType::RArray(vector_aux);
                // le pasamos el request al command_service
                let commander = &mut shared_commander.lock().unwrap(); //handle error
                commander.handle_command(&operation, client_addrs);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    //lo de abajo es para que clippy no se queje

    let algo_1 = RespType::RError("no hay error".to_string());
    let algo_2 = RespType::RNullBulkString();
    let algo_3 = RespType::RNullArray();
    let algo_4 = RespType::RInteger(2);
    let algo_5 = RespType::RSimpleString("corto".to_string());
    println!("{:?}", algo_1);
    println!("{:?}", algo_2);
    println!("{:?}", algo_3);
    println!("{:?}", algo_4);
    println!("{:?}", algo_5);
}
