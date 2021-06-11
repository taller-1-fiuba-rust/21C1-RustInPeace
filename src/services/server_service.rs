//#[derive(Debug)]
use super::parser_service::{parse_request, parse_response};
// use super::utils::resp_type::RespType;
use super::worker_service::ThreadPool;
use crate::domain::entities::config::Config;
use crate::domain::entities::message::WorkerMessage;
use crate::domain::entities::server::Server;
use crate::domain::implementations::database::Database;
use crate::services::commander::handle_command;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, RwLock};
// use std::time::Duration;

pub fn init(server: &mut Server, db: Database, config: Config) {
    let database = Arc::new(RwLock::new(db));
    let conf = Arc::new(RwLock::new(config));

    let (sender_server, receiver_server) = mpsc::channel();
    let port: &String = server.get_port();
    let dir: &String = server.get_dir();
    let threadpool_size: &usize = server.get_threadpool_size();
    let pool = ThreadPool::new(*threadpool_size);
    match TcpListener::bind(format!("{}:{}", dir, port)) {
        Ok(listener) => {
            'outer: for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        //if timeout != 0 {
                        //stream.set_read_timeout(Some(Duration::from_millis(timeout)));//handle err
                        //}
                        // stream.set_nonblocking(true).expect("set_nonblocking call failed");

                        let tx = sender_server.clone();
                        let conf_lock = conf.clone();
                        let cloned_database = database.clone();
                        pool.spawn(|| {
                            handle_connection(stream, tx, cloned_database, conf_lock);
                        });

                        for msg in &receiver_server {
                            match msg {
                                WorkerMessage::Log(log_msg) => match server.log(log_msg) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        println!("Logging error: {}", e);
                                    }
                                },
                                WorkerMessage::Verb(verbose_txt) => {
                                    server.verbose(verbose_txt);
                                }
                                WorkerMessage::NewOperation(operation, addrs) => {
                                    server.update_clients_operations(operation, addrs);
                                }
                                WorkerMessage::MonitorOp(addrs) => {
                                    server.print_last_operations_by_client(addrs);
                                }
                                WorkerMessage::Shutdown => {
                                    println!("SHUTDOWN!");
                                    break 'outer;
                                }
                                WorkerMessage::HandleNextMessage => {
                                    break;
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

fn handle_connection(
    mut stream: TcpStream,
    tx: Sender<WorkerMessage>,
    database: Arc<RwLock<Database>>,
    config: Arc<RwLock<Config>>,
) {
    let client_addrs = stream.peer_addr().unwrap();
    tx.send(WorkerMessage::Log(format!(
        "Connection to address {} established\r\n",
        client_addrs
    )))
    .unwrap();

    // leo un mensaje nuevo
    // el mensaje nuevo llega en un arreglo de bytes
    // se lo pasamos a parser
    // stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();

    let mut buf = [0u8; 512];

    match stream.read(&mut buf) {
        Ok(0) => {
            return;
        }
        Ok(size) => {
            tx.send(WorkerMessage::Log(format!(
                "Reading new message from {}. Message: {:?}\r\n",
                client_addrs,
                String::from_utf8_lossy(&buf[..size])
            )))
            .unwrap();
            tx.send(WorkerMessage::Verb(format!(
                "Reading new message from {}. Message: {:?}\r\n",
                client_addrs,
                String::from_utf8_lossy(&buf[..size])
            )))
            .unwrap();

            match parse_request(&buf[..size]) {
                Ok(parsed_request) => {
                    tx.send(WorkerMessage::Log(format!(
                        "Parsed request: {:?}\r\n",
                        parsed_request
                    )))
                    .unwrap();
                    tx.send(WorkerMessage::Verb(format!(
                        "Parsed request: {:?}\r\n",
                        parsed_request
                    )))
                    .unwrap();
                    // de aca pasa a un servicio que delegue segun la request que sea
                    tx.send(WorkerMessage::NewOperation(
                        parsed_request.clone(),
                        client_addrs,
                    ))
                    .unwrap();
                    // le pasamos el request al commander
                    if let Some(res) =
                        handle_command(parsed_request, &tx, client_addrs, &database, &config)
                    {
                        // ese servicio va a devolver una response
                        // simulo una response:
                        let response = parse_response(res);

                        tx.send(WorkerMessage::Verb(format!(
                            "Response for {}. Message: {:?}. Response: {}\r\n",
                            client_addrs,
                            String::from_utf8_lossy(&buf[..size]),
                            response
                        )))
                        .unwrap();

                        tx.send(WorkerMessage::Log(format!(
                            "Response for {}. Message: {:?}. Response: {}\r\n",
                            client_addrs,
                            String::from_utf8_lossy(&buf[..size]),
                            response
                        )))
                        .unwrap();
                        println!("RESPONSE: {}", response);
                        println!("RESPONSE as bytes: {:?}", response.as_bytes());
                        stream.write_all(response.as_bytes()).unwrap();
                        stream.flush().unwrap();
                    }
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Error al leer: {:?}", e);
        }
    }

    tx.send(WorkerMessage::HandleNextMessage).unwrap();

    //lo de abajo es para que clippy no se queje

    // let algo_1 = RespType::RError("no hay error".to_string());
    // let algo_2 = RespType::RNullBulkString();
    // let algo_3 = RespType::RNullArray();
    // let algo_4 = RespType::RInteger(2);
    // let algo_5 = RespType::RSimpleString("corto".to_string());
    // println!("{:?}", algo_1);
    // println!("{:?}", algo_2);
    // println!("{:?}", algo_3);
    // println!("{:?}", algo_4);
    // println!("{:?}", algo_5);
}
