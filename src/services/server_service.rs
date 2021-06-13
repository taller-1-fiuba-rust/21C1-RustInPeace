//#[derive(Debug)]
use super::parser_service::{parse_request, parse_response};
use super::worker_service::ThreadPool;
use crate::domain::entities::config::Config;
use crate::domain::entities::message::WorkerMessage;
use crate::domain::entities::server::Server;
use crate::domain::implementations::database::Database;
use crate::services::commander::handle_command;
use crate::services::utils::resp_type::RespType;
use std::io::{self, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
// use std::time::Duration;

pub fn init(server: &mut Server, db: Database, config: Config) {
    let (sender_server, receiver_server) = mpsc::channel();
    let port: String = server.get_port().clone();
    let dir: String = server.get_dir().clone();
    let threadpool_size: usize = *server.get_threadpool_size();
    let pool = ThreadPool::new(threadpool_size);

    let handle: thread::JoinHandle<()> = thread::spawn(move || {
        let database = Arc::new(RwLock::new(db));
        let conf = Arc::new(RwLock::new(config));
        let (stop_signal_sender, stop_signal_receiver) = mpsc::channel();

        match TcpListener::bind(format!("{}:{}", dir, port)) {
            Ok(listener) => {
                listener.set_nonblocking(true).expect("non block error");
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => {
                            //if timeout != 0 {
                            //stream.set_read_timeout(Some(Duration::from_millis(timeout)));//handle err
                            //}
                            stream
                                .set_nonblocking(true)
                                .expect("set_nonblocking call failed");

                            let tx = sender_server.clone();
                            let conf_lock = conf.clone();
                            let cloned_database = database.clone();
                            let stop = stop_signal_sender.clone();
                            pool.spawn(|| {
                                handle_connection(stream, tx, cloned_database, conf_lock, stop);
                            });
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            // Decide if we should exit
                            if let Ok(drop) = stop_signal_receiver.try_recv() {
                                if drop {
                                    break;
                                }
                            }
                            // Decide if we should try to accept a connection again
                            continue;
                        }
                        Err(_) => {
                            println!("Couldn't get stream");
                            continue;
                        }
                    }
                }
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Listener couldn't be created");
            }
        }
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
        }
    }
    println!("Shutting down.");
    handle.join().unwrap();
}

fn handle_connection(
    mut stream: TcpStream,
    tx: Sender<WorkerMessage>,
    database: Arc<RwLock<Database>>,
    config: Arc<RwLock<Config>>,
    stop: Sender<bool>,
) {
    let client_addrs = stream.peer_addr().unwrap();
    // println!("HOLISSS SOY {}", client_addrs);

    tx.send(WorkerMessage::Log(format!(
        "Connection to address {} established\r\n",
        client_addrs
    )))
    .unwrap();

    // leo un mensaje nuevo
    // el mensaje nuevo llega en un arreglo de bytes
    // se lo pasamos a parser
    // stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();

    // let mut buf = [0u8; 512];
    loop {
        let mut buf = [0u8; 512];
        match stream.read(&mut buf) {
            Ok(0) => {
                break;
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
                        // println!("parsed req: {:?} from: {}", parsed_request, client_addrs);
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

                        //chequeo si es un shutdown
                        if shutdown(&parsed_request) {
                            stop.send(true).unwrap();
                            break;
                        }

                        // le pasamos el request al commander
                        if let Some(res) =
                            handle_command(parsed_request, &tx, client_addrs, &database, &config)
                        {
                            // ese servicio va a devolver una response
                            let response = parse_response(res);
                            // println!("response:{:?}", response);
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
                            // println!("RESPONSE: {}", response);
                            // println!("RESPONSE as bytes: {:?}", response.as_bytes());
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                        }
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        break;
                    }
                }
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                println!("read is blocking..");
            }
            Err(e) => {
                println!("Closing connection: {:?}", e);
                break;
            }
        }
    }
}

fn shutdown(request: &RespType) -> bool {
    if let RespType::RArray(array) = request {
        if let RespType::RBulkString(cmd) = &array[0] {
            if cmd == "shutdown" {
                return true;
            }
        }
    }
    false
}
