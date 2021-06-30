// use crate::domain::implementations::database::Database;
use crate::domain::implementations::logger_impl::Logger;
use crate::domain::implementations::operation_register_impl::OperationRegister;
use crate::services::utils::resp_type::RespType;
// use crate::services::worker_service::ThreadPool;
use std::collections::HashMap;
// use std::io::Read;
// use std::io::Write;
// use std::net::TcpListener;
// use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
// use std::sync::RwLock;
// use std::sync::atomic::AtomicUsize;
// use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
// use std::thread;
use std::{io::Error, net::SocketAddr};

// use super::config::Config;
use super::message::WorkerMessage;

// pub struct Client {
//     address: SocketAddr,
//     operations: OperationRegister,
//     subscriptions: Vec<String>
// }

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
    receiver: Arc<Mutex<mpsc::Receiver<WorkerMessage>>>,
}

impl Server {
    pub fn new(
        port: String,
        logfile: String,
        verb: String,
        receiver: Arc<Mutex<Receiver<WorkerMessage>>>,
    ) -> Result<Self, Error> {
        let dir = "127.0.0.1".to_string();
        // let threadpool_size = 4;
        let port = port;
        let verbose = verb;
        let receiver = receiver;
        let logger_path = &logfile;
        let logger = Logger::new(logger_path)?;
        let clients_operations = HashMap::new();
        let channels = HashMap::new();

        Ok(Server {
            dir,
            port,
            verbose,
            logger,
            clients_operations,
            channels,
            receiver,
        })
    }

    /// Escucha mensajes provenientes de los workers, según el mensaje delega al server una tarea distinta.
    /// Las tareas pueden ser: log, verbose, update_clients_operation, print_last_operations_by_client,
    /// subscribe, stop, unsubscribe, unsubscribeall, publish
    pub fn listen(&mut self) {
        loop {
            let msg = self.receiver.lock().unwrap().recv().unwrap();
            match msg {
                WorkerMessage::Log(log_msg) => match self.log(log_msg) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("Logging error: {}", e);
                    }
                },
                WorkerMessage::MonitorOp(addrs) => {
                    self.print_last_operations_by_client(addrs);
                }
                WorkerMessage::NewOperation(operation, addrs) => {
                    self.update_clients_operations(operation, addrs);
                }
                WorkerMessage::Stop(_) => {
                    break;
                }
                WorkerMessage::Verb(verbose_txt) => {
                    self.verbose(verbose_txt);
                }
                WorkerMessage::Subscribe(channel, addrs, message_sender) => {
                    self.subscribe_to_channel(channel, addrs, message_sender);
                }
                WorkerMessage::Unsubscribe(channel, addrs) => {
                    self.unsubscribe(channel, addrs);
                }
                WorkerMessage::UnsubscribeAll(addrs) => {
                    self.unsubscribe_to_all_channels(addrs);
                }
                WorkerMessage::Publish(channel, response_sender, message) => {
                    let messages_sent = self.send_message_to_channel(channel, message);
                    response_sender.send(messages_sent).unwrap();
                }
            }
        }
    }

    /// Retorna el puerto donde escucha el server
    pub fn get_port(&self) -> &String {
        &self.port
    }

    /// Retorna la dirección IP del server
    pub fn get_dir(&self) -> &String {
        &self.dir
    }

    /// Retorna un String "1" si verbose es true, "0" sino.
    /// Verbose true implica imprimir mensajes que describan lo que pasa en el server
    pub fn get_verbose(&self) -> &String {
        &self.verbose
    }

    // pub fn get_threadpool_size(&self) -> &usize {
    //     &self.threadpool_size
    // }

    /// Envia un mensaje al Logger para que lo imprima en el archivo de logs
    pub fn log(&mut self, msg: String) -> Result<(), Error> {
        self.logger.log(msg.as_bytes())?;
        Ok(())
    }

    /// Si verbose es 1 (true), imprime el mensaje recibido
    pub fn verbose(&self, msg: String) {
        if self.parse_verbose(self.get_verbose()) == 1 {
            println!("{}", msg);
        }
    }

    /// Convierte el verbose original de tipo String a tipo Usize
    fn parse_verbose(&self, string: &str) -> usize {
        let mut verbose: usize = 1;
        let verb_aux = string.parse::<usize>();
        match verb_aux {
            Ok(verb) => verbose = verb,
            Err(_) => println!("parsing error"),
        }
        verbose
    }

    /// Recibe una nueva operación de un cliente y la agrega a la lista de operaciones
    /// Busca al cliente por dirección. Si es un cliente nuevo, primero lo agrega con un OperationRegister vacio
    /// luego agrega la nueva operacion
    pub fn update_clients_operations(&mut self, operation: RespType, addrs: SocketAddr) {
        let last_operations = self
            .clients_operations
            .entry(addrs.to_string())
            .or_insert_with(|| OperationRegister::new(100));
        last_operations.store_operation(operation);
    }

    //
    pub fn print_last_operations_by_client(&self, addrs: String) {
        if let Some(operations) = self.clients_operations.get(&addrs) {
            for operation in operations.get_operations() {
                println!("{:?}", operation)
            }
        }
    }

    /// Suscribe un cliente al channel
    /// Primero chequea si el channel ya existe, si existe agrega al cliente
    /// Sino lo crea y agrega al cliente y su sender
    pub fn subscribe_to_channel(
        &mut self,
        channel: String,
        addrs: SocketAddr,
        sender: Sender<String>,
    ) {
        if let Some(subscribers) = self.channels.get_mut(&channel) {
            subscribers.insert(addrs.ip().to_string(), sender);
        } else {
            let mut inner_map = HashMap::new();
            inner_map.insert(addrs.ip().to_string(), sender);
            self.channels.entry(channel).or_insert(inner_map);
        }
    }

    /// Desuscribe la dirección dada de todos los canales a los que este suscrito
    /// Por el sender asociado envia mensaje para dejar de aceptar
    pub fn unsubscribe_to_all_channels(&mut self, addrs: SocketAddr) {
        for subscriber in self.channels.values_mut() {
            match subscriber.get(&addrs.ip().to_string()) {
                Some(sender) => {
                    sender.send(String::from("UNSUBSCRIBE")).unwrap();
                    sender.send(String::from("QUIT")).unwrap();
                    subscriber.remove(&addrs.ip().to_string());
                }
                None => break,
            }
        }
    }

    /// Desuscribe la dirección addrs del canal
    /// Elimina la dirección del hashmap de suscriptores de dicho canal
    pub fn unsubscribe(&mut self, channel: String, addrs: SocketAddr) {
        let subscribers = self.channels.get_mut(&channel).unwrap();
        let sender = subscribers.get(&addrs.ip().to_string()).unwrap();
        sender.send(String::from("UNSUBSCRIBE")).unwrap();
        sender.send(String::from("QUIT")).unwrap(); //esto deberia pasar si la direccion no tiene ninguna suscripcion
        subscribers.remove(&addrs.ip().to_string());
    }

    /// Envia el mensaje msg a todas las direcciones asociadas al canal dado
    pub fn send_message_to_channel(&mut self, channel: String, msg: String) -> usize {
        let mut sent = 0;
        let subscribers = self.channels.get_mut(&channel).unwrap();
        for sender in subscribers.values() {
            match sender.send(msg.clone()) {
                Ok(()) => sent += 1,
                Err(_) => continue,
            }
        }
        sent
    }
}

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
