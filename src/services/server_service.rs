//#[derive(Debug)]
use super::parser_service::{parse_request, parse_response};
use super::utils::resp_type::RespType;
use super::worker_service::ThreadPool;
use crate::domain::entities::message::WorkerMessage;
use crate::domain::entities::server::Server;
use crate::services::commander::handle_command;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{self, Sender};
use std::time::Duration;
use crate::domain::implementations::database::Database;

pub fn init(server: &mut Server, db: Database) {
    // let db=server.get_database();
    let mut database = Arc::new(RwLock::new(db));
    //let probando = *database;
    //let storage = Arc::new(storage);
    let (sender_server, receiver_server) = mpsc::channel();
    let port: &String = server.get_port();
    let dir: &String = server.get_dir();
    let threadpool_size: &usize = server.get_threadpool_size();
    let pool = ThreadPool::new(*threadpool_size);
    match TcpListener::bind(format!("{}:{}", dir, port)) {
        Ok(listener) => {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        // match stream.peer_addr() {
                        //     Ok(addrs) => {
                        //         println!("New stream: {}", addrs);
                        //     }
                        //     Err(_) => {
                        //         println!("Could't get client's address");
                        //     }
                        // }
                        //if timeout != 0 {
                        //stream.set_read_timeout(Some(Duration::from_millis(timeout)));//handle err
                        //}
                        //let shared_commander = Arc::clone(&commander);
                        let tx = sender_server.clone();
                        //let cloned_database = database.clone();
                        // let server_cloned = server.clone();
                        let cloned_database = database.clone();
                        pool.spawn(move || {
                            handle_connection(stream, tx, cloned_database); //, shared_commander);
                        });
                        println!("llegamosssssssssssss");

                        for msg in &receiver_server {
                            match msg {
                                WorkerMessage::Log(log_msg) => match server.log(log_msg) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        println!("Logging error: {}", e);
                                    }
                                },
                                WorkerMessage::NewOperation(operation, addrs) => {
                                    server.update_clients_operations(operation, addrs);
                                }
                                WorkerMessage::MonitorOp(addrs) => {
                                    server.print_last_operations_by_client(addrs);
                                }
                            }
                        }
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

fn handle_connection(stream: TcpStream, tx: Sender<WorkerMessage>, database: Arc<RwLock<Database>>) {
    std::thread::sleep(Duration::from_secs(2));
    let client_addrs = stream.peer_addr().unwrap();
    tx.send(WorkerMessage::Log(format!(
        "Connection to address {} established",
        client_addrs
    )))
    .unwrap();
    // leo un mensaje nuevo
    // el mensaje nuevo llega en un arreglo de bytes
    // se lo pasamos a parser
    let reader = BufReader::new(stream);
    let lines = reader.lines();
    // iteramos las lineas que recibimos de nuestro cliente
    for line in lines {
        let bytes = line.unwrap();
        tx.send(WorkerMessage::Log(format!(
            "Reading new message from {}. Message: {:?}",
            client_addrs, &bytes
        )))
        .unwrap();
        let message = bytes.as_bytes();
        println!("recibido as bytes: {:?}", message);
        match parse_request(message) {
            Ok(parsed_request) => {
                println!("Parsed request: {:?}", parsed_request);
                // de aca pasa a un servicio que delegue segun la request que sea
                // simulo request parseada:
                let elemento_1 = RespType::RBulkString("monitor".to_string());
                let vector_aux = vec![elemento_1];
                let operation = RespType::RArray(vector_aux);
                tx.send(WorkerMessage::NewOperation(operation.clone(), client_addrs))
                    .unwrap();
                // le pasamos el request al command_service
                // let mut commander = Commander::new(); //&mut shared_commander.lock().unwrap(); //handle error
                handle_command(operation, &tx, client_addrs,&database);

                // ese servicio va a devolver una response
                // simulo una response:
                let response = parse_response(RespType::RInteger(5));
                println!("Parsed response: {}", response);
                tx.send(WorkerMessage::Log(format!(
                    "Response for {}. Message: {:?}. Response: {}",
                    client_addrs, &bytes, response
                )))
                .unwrap();
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
