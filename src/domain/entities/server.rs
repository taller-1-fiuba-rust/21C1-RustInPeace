use crate::domain::implementations::database::Database;
use crate::domain::implementations::logger_impl::Logger;
use crate::domain::implementations::operation_register_impl::OperationRegister;
use crate::services::commander::handle_command;
use crate::services::parser_service::parse_request;
use crate::services::parser_service::parse_response;
use crate::services::server_service::handle_connection;
use crate::services::utils::resp_type::RespType;
use crate::services::worker_service::ThreadPool;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::{io::Error, net::SocketAddr};

use super::config::Config;
use super::message::WorkerMessage;

#[derive(Debug)]
pub struct Server {
    dir: String,
    port: String,
    verbose: String,
    // threadpool_size: usize,
    logger: Logger, // receiver: Arc<Mutex<mpsc::Receiver<WorkerMessage>>>
    clients_operations: HashMap<String, OperationRegister>,
    channels: HashMap<String, HashMap<String, Sender<String>>>,
    // threadpool: ThreadPool,
    receiver: Arc<Mutex<mpsc::Receiver<WorkerMessage>>>
}

impl Server {
    pub fn new(port: String, logfile: String, verb: String, receiver: Arc<Mutex<Receiver<WorkerMessage>>>) -> Result<Self, Error> {
        let dir = "127.0.0.1".to_string();
        // let threadpool_size = 4;
        let port = port;
        let verbose = verb;
        let receiver = receiver;
        let logger_path = &logfile;
        let logger = Logger::new(logger_path)?;
        let clients_operations = HashMap::new();
        let channels = HashMap::new();
        // let threadpool = ThreadPool::new(4);
        // let server = ServerImpl::new(port, logfile, verb);

        // let _ = thread::spawn(|| loop {
        //     let request = receiver.lock().unwrap().recv();
        // });

        Ok(Server {
            dir,
            port,
            verbose,
            // threadpool_size,
            logger,
            clients_operations,
            channels,
            // threadpool,
            receiver
        })
    }



    // pub fn handle_request(&mut self, receiver: &Receiver<WorkerMessage>) {
    //     if let Ok(request) = receiver.recv() {
    //         if let WorkerMessage::Request(stream, tx, db, config, stop) = request {
    //             // let (sender_server, receiver_server) = mpsc::channel();
    //             self.threadpool.spawn(||
    //                 handle_connection(stream, tx, db, config, stop)
    //             );
    //             // self.listen(receiver);
    //         }
    //     }
    // }

    pub fn listen(&mut self) {
        loop {
            let msg = self.receiver.lock().unwrap().recv().unwrap();
            match msg {
                WorkerMessage::Log(log_msg) => {
                    match self.log(log_msg) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Logging error: {}", e);
                        }
                    }
                }
                WorkerMessage::MonitorOp(addrs) => {
                    self.print_last_operations_by_client(addrs);
                }
                WorkerMessage::NewOperation(operation, addrs) => {
                    self.update_clients_operations(operation, addrs);
                }
                // WorkerMessage::Request(_) => {}
                WorkerMessage::Stop(_) => {
                    break;
                }
                WorkerMessage::Verb(verbose_txt) => {
                    self.verbose(verbose_txt);
                }
                WorkerMessage::Subscribe(channel, addrs, message_sender) => {
                    let mut inner_map = HashMap::new();
                    inner_map.insert(addrs.to_string(), message_sender);
                    self.channels.entry(channel).or_insert(inner_map);
                }
            }
        }
    }

    // pub fn listen(&mut self, receiver: &Receiver<WorkerMessage>) {
    //     for msg in receiver {
    //         println!("msg: {:?}", msg);
    //         match msg {
    //             WorkerMessage::Log(log_msg) => match self.log(log_msg) {
    //                 Ok(_) => (),
    //                 Err(e) => {
    //                     println!("Logging error: {}", e);
    //                 }
    //             },
    //             WorkerMessage::Verb(verbose_txt) => {
    //                 self.verbose(verbose_txt);
    //             }
    //             WorkerMessage::NewOperation(operation, addrs) => {
    //                 self.update_clients_operations(operation, addrs);
    //             }
    //             WorkerMessage::MonitorOp(addrs) => {
    //                 self.print_last_operations_by_client(addrs);
    //             }
    //             WorkerMessage::Stop(_) => {
    //                 break;
    //             }
    //             _ => continue
    //         }
    //     }
    // }

    pub fn get_port(&self) -> &String {
        &self.port
    }

    pub fn get_dir(&self) -> &String {
        &self.dir
    }

    pub fn get_verbose(&self) -> &String {
        &self.verbose
    }

    // pub fn get_threadpool_size(&self) -> &usize {
    //     &self.threadpool_size
    // }

    pub fn log(&mut self, msg: String) -> Result<(), Error> {
        self.logger.log(msg.as_bytes())?;
        Ok(())
    }

    pub fn verbose(&self, msg: String) {
        if self.parse_verbose(self.get_verbose()) == 1 {
            println!("{}", msg);
        }
    }

    fn parse_verbose(&self, string: &str) -> usize {
        let mut verbose: usize = 1;
        let verb_aux = string.parse::<usize>();
        match verb_aux {
            Ok(verb) => verbose = verb,
            Err(_) => println!("parsing error"),
        }
        verbose
    }

    pub fn update_clients_operations(&mut self, operation: RespType, addrs: SocketAddr) {
        let last_operations = self
            .clients_operations
            .entry(addrs.to_string())
            .or_insert_with(|| OperationRegister::new(100));
        last_operations.store_operation(operation);
    }

    pub fn print_last_operations_by_client(&self, addrs: String) {
        if let Some(operations) = self.clients_operations.get(&addrs) {
            for operation in operations.get_operations() {
                println!("{:?}", operation)
            }
        }
    }

    // pub fn init(self, db: Database, config: Config) {
    //     let (sender_server, receiver_server) = mpsc::channel();
    //     let port: String = self.port.clone();
    //     let dir: String = self.dir.clone();
    //     // let threadpool_size: usize = *server.get_threadpool_size();
    //     // let pool = ThreadPool::new(threadpool_size);
    
    //     // let handle: thread::JoinHandle<()> = thread::spawn(move || {
    //         let database = Arc::new(RwLock::new(db));
    //         let conf = Arc::new(RwLock::new(config));
    //         let (stop_signal_sender, stop_signal_receiver) = mpsc::channel();
    
    //         match TcpListener::bind(format!("{}:{}", dir, port)) {
    //             Ok(listener) => {
    //                 for stream in listener.incoming() {
    //                     match stream {
    //                         Ok(stream) => {
    //                             let tx = sender_server.clone();
    //                             let conf_lock = conf.clone();
    //                             let cloned_database = database.clone();
    //                             let stop = stop_signal_sender.clone();
    //                             self.threadpool.spawn(|| {
    //                                 handle_connection(stream, tx, cloned_database, conf_lock, stop);
    //                             });
    //                             if let Ok(drop) = stop_signal_receiver.recv() {
    //                                 if drop {
    //                                     save_database(database);
    //                                     //stop server listener
    //                                     break;
    //                                 }
    //                             }
    //                         }
    //                         Err(_) => {
    //                             println!("Couldn't get stream");
    //                             continue;
    //                         }
    //                     }
    //                 }
    //             }
    //             Err(_) => {
    //                 println!("Listener couldn't be created");
    //             }
    //         }
    //     // });
    //     // listen_server_messages(receiver_server, server);
    //     println!("Shutting down.");
    //     // handle.join().unwrap();
    // }
}

pub struct ServerImpl {
    port: String,
    verbose: String,
    logger: Logger,
    clients_operations: HashMap<String, OperationRegister>,
    dir: String
}

impl ServerImpl {
    pub fn new(port: String, logfile: String, verb: String) -> Result<Self, Error> {
        let dir = "127.0.0.1".to_string();
        let port = port;
        let verbose = verb;
        let logger_path = &logfile;
        let logger = Logger::new(logger_path)?;
        let clients_operations = HashMap::new();

        Ok(ServerImpl {
            port,
            verbose,
            logger,
            clients_operations,
            dir
        })
    }

    pub fn get_port(&self) -> &String {
        &self.port
    }

    pub fn get_dir(&self) -> &String {
        &self.dir
    }

    pub fn get_verbose(&self) -> &String {
        &self.verbose
    }

    pub fn log(&mut self, msg: String) -> Result<(), Error> {
        self.logger.log(msg.as_bytes())?;
        Ok(())
    }

    pub fn verbose(&self, msg: String) {
        if self.parse_verbose(self.get_verbose()) == 1 {
            println!("{}", msg);
        }
    }

    fn parse_verbose(&self, string: &str) -> usize {
        let mut verbose: usize = 1;
        let verb_aux = string.parse::<usize>();
        match verb_aux {
            Ok(verb) => verbose = verb,
            Err(_) => println!("parsing error"),
        }
        verbose
    }

    pub fn update_clients_operations(&mut self, operation: RespType, addrs: SocketAddr) {
        let last_operations = self
            .clients_operations
            .entry(addrs.to_string())
            .or_insert_with(|| OperationRegister::new(100));
        last_operations.store_operation(operation);
    }

    pub fn print_last_operations_by_client(&self, addrs: String) {
        if let Some(operations) = self.clients_operations.get(&addrs) {
            for operation in operations.get_operations() {
                println!("{:?}", operation)
            }
        }
    }
}


// fn handle_connection(
//     mut stream: TcpStream,
//     tx: Sender<WorkerMessage>,
//     database: Arc<RwLock<Database>>,
//     config: Arc<RwLock<Config>>,
//     stop: Sender<bool>,
// ) {
//     let client_addrs = stream.peer_addr().unwrap();
//     log(
//         format!("Connection to address {} established\r\n", client_addrs),
//         &tx,
//     );

//     // stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
//     loop {
//         let mut buf = [0u8; 512];
//         match stream.read(&mut buf) {
//             Ok(0) => {
//                 break;
//             }
//             Ok(size) => {
//                 log(
//                     format!(
//                         "Reading new message from {}. Message: {:?}\r\n",
//                         client_addrs,
//                         String::from_utf8_lossy(&buf[..size])
//                     ),
//                     &tx,
//                 );

//                 match parse_request(&buf[..size]) {
//                     Ok(parsed_request) => {
//                         log(format!("Parsed request: {:?}\r\n", parsed_request), &tx);

//                         tx.send(WorkerMessage::NewOperation(
//                             parsed_request.clone(),
//                             client_addrs,
//                         ))
//                         .unwrap();

//                         if check_shutdown(&parsed_request) {
//                             // stop.send(true).unwrap();
//                             tx.send(WorkerMessage::Stop(true)).unwrap();
//                             break;
//                         }

//                         if let Some(res) =
//                             handle_command(parsed_request, &tx, client_addrs, &database, &config)
//                         {
//                             let response = parse_response(res);
//                             log(
//                                 format!(
//                                     "Response for {}. Message: {:?}. Response: {}\r\n",
//                                     client_addrs,
//                                     String::from_utf8_lossy(&buf[..size]),
//                                     response
//                                 ),
//                                 &tx,
//                             );

//                             stream.write_all(response.as_bytes()).unwrap();
//                             stream.flush().unwrap();
//                             // stop.send(false).unwrap();
//                             tx.send(WorkerMessage::Stop(false)).unwrap();
//                         }
//                     }
//                     Err(e) => {
//                         println!("Error trying to parse request: {:?}", e);
//                         continue;
//                     }
//                 }
//             }
//             // Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
//             Err(e) => {
//                 println!("Closing connection: {:?}", e);
//                 break;
//             }
//         }
//     }
// }

// /// Recibe una base de datos de tipo Database protegida por un RwLock
// /// y guarda la información en su correspondiente archivo
// fn save_database(database: Arc<RwLock<Database>>) {
//     println!("Saving dump before shutting down");
//     let x = Arc::try_unwrap(database);
//     match x {
//         Ok(t) => {
//             match t.try_read() {
//                 Ok(n) => n._save_items_to_file(),
//                 Err(_) => unreachable!(),
//             };
//         }
//         Err(_) => {
//             println!("Database couldn't be saved into file");
//         }
//     }
// }

// /// Recibe un mensaje msg de tipo String y un sender tx de mensajes de tipo WorkerMessage
// /// El sender envia el mensaje Log
// fn log(msg: String, tx: &Sender<WorkerMessage>) {
//     tx.send(WorkerMessage::Log(msg)).unwrap();
// }

// /// Recibe un mensaje msg de tipo String y un sender tx de mensajes de tipo WorkerMessage
// /// El sender envia el mensaje Verbose
// fn _verbose(msg: String, tx: &Sender<WorkerMessage>) {
//     tx.send(WorkerMessage::Verb(msg)).unwrap();
// }

// /// Recibe una solicitud request de tipo &RespType y valida si es el comando "SHUTDOWN"
// /// Devuelve true si lo es, false si no
// fn check_shutdown(request: &RespType) -> bool {
//     if let RespType::RArray(array) = request {
//         if let RespType::RBulkString(cmd) = &array[0] {
//             if cmd == "shutdown" {
//                 return true;
//             }
//         }
//     }
//     false
// }




// #[test]
// fn test_01_se_guarda_una_operacion_de_tipo_info_en_operation_register() {
//     // use super::config::Config;
//     use super::server::Server;
//     use std::net::{IpAddr, Ipv4Addr};

//     // let verbose = 0;
//     // let timeout = 0;
//     let port = "8080".to_string();
//     let verbose = "1".to_string();
//     let logfile = "./src/dummy_1.log".to_string();

//     let mut server = Server::new(port, logfile, verbose).unwrap();
//     let dummy_operation = RespType::RArray(vec![RespType::RBulkString(String::from("info"))]);
//     let mut operation_register = OperationRegister::new(100);
//     operation_register.store_operation(dummy_operation.clone());

//     let dir = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
//     server.update_clients_operations(dummy_operation, dir);
//     let saved_operations = server.clients_operations.get(&dir.to_string()).unwrap();
//     assert_eq!(
//         saved_operations.get_operations(),
//         operation_register.get_operations()
//     );

//     std::fs::remove_file("./src/dummy_1.log").unwrap();
// }

// #[test]
// fn test_02_se_guardan_multiples_operaciones_en_register_operation() {
//     // use super::config::Config;
//     use super::server::Server;
//     use std::net::{IpAddr, Ipv4Addr};

//     // let verbose = 0;
//     // let timeout = 0;
//     let port = "8080".to_string();
//     let verbose = "1".to_string();
//     let logfile = "./src/dummy.log".to_string();

//     let mut server = Server::new(port, logfile, verbose).unwrap();
//     let dummy_operation = RespType::RArray(vec![RespType::RBulkString(String::from("info"))]);
//     let dummy_operation_2 = RespType::RArray(vec![
//         RespType::RBulkString(String::from("set")),
//         RespType::RBulkString(String::from("key")),
//         RespType::RBulkString(String::from("value")),
//     ]);

//     let mut operation_register = OperationRegister::new(100);
//     operation_register.store_operation(dummy_operation.clone());
//     operation_register.store_operation(dummy_operation_2.clone());

//     let dir = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
//     server.update_clients_operations(dummy_operation, dir);
//     server.update_clients_operations(dummy_operation_2, dir);

//     let saved_operations = server.clients_operations.get(&dir.to_string()).unwrap();
//     assert_eq!(
//         saved_operations.get_operations(),
//         operation_register.get_operations()
//     );

//     std::fs::remove_file("./src/dummy.log").unwrap();
// }
