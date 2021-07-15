use super::client::Client;
use super::message::WorkerMessage;
use crate::domain::implementations::logger_impl::Logger;
use crate::domain::implementations::operation_register_impl::OperationRegister;
use crate::services::parser_service;
use crate::services::utils::glob_pattern;
use crate::services::utils::resp_type::RespType;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::usize;
use std::{io::Error, net::SocketAddr};

#[derive(Debug)]
pub struct Server {
    dir: String,
    port: String,
    verbose: String,
    logger: Logger,
    clients: Vec<Client>,
    // clients_operations: HashMap<String, OperationRegister>,
    channels: HashMap<String, HashMap<String, (Sender<usize>, TcpStream)>>,
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
        let port = port;
        let verbose = verb;
        let receiver = receiver;
        let logger_path = &logfile;
        let logger = Logger::new(logger_path)?;
        // let clients_operations = HashMap::new();
        //tener canales, publicadores y subscriptores
        let channels = HashMap::new();
        let clients = Vec::new();

        Ok(Server {
            dir,
            port,
            verbose,
            logger,
            clients,
            // clients_operations,
            channels,
            receiver,
        })
    }

    /// Escucha mensajes provenientes de los workers, según el mensaje delega al server una tarea distinta.
    ///
    /// Las tareas pueden ser:
    /// * Log: escribe un mensaje en el archivo log, cuya direccion está definida en el archivo de config.
    /// * Verb: imprime mensajes por consola, indicando el funcionamiento interno del servidor.
    /// * NewOperation: registra el ultimo comando ingresado por el cliente.
    /// * MonitorOp: devuelve todas las operaciones registradas por el servidor.
    /// * Subscribe: suscribe al cliente a un canal dado.
    /// * stop
    /// * Unsubscribe: desuscribe al cliente del canal dado.
    /// * UnsubscribeAll: desuscribe al cliente de todos los canales a los que se haya suscrito.
    /// * Publish: publica un mensaje en los canales especificados.
    /// * Channels: lista canales activos.
    /// * Numsub: lista cantidad de suscriptores por canal.
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
                WorkerMessage::SetMonitor(addrs) => {
                    // self.print_last_operations_by_client(stream);
                    self.set_client_to_monitor_state(addrs);
                }
                WorkerMessage::AddClient(client) => {
                    self.clients.push(client);
                }
                WorkerMessage::NewOperation(operation, addrs) => {
                    self.check_monitor(operation, addrs);
                }
                WorkerMessage::Stop(_) => {
                    break;
                }
                WorkerMessage::Verb(verbose_txt) => {
                    self.verbose(verbose_txt);
                }
                WorkerMessage::Subscribe(channel, addrs, message_sender, stream) => {
                    self.subscribe_to_channel(channel, addrs, message_sender, stream);
                }
                WorkerMessage::Unsubscribe(channel, addrs, message_sender) => {
                    self.unsubscribe(channel, addrs, message_sender);
                }
                WorkerMessage::UnsubscribeAll(addrs, message_sender) => {
                    self.unsubscribe_to_all_channels(addrs, message_sender);
                }
                WorkerMessage::Publish(channel, response_sender, message) => {
                    let messages_sent = self.send_message_to_channel(channel, message);
                    response_sender.send(messages_sent).unwrap();
                }
                WorkerMessage::Channels(response_sender, pattern) => {
                    if let Some(pattern) = pattern {
                        self.list_active_channels_by_pattern(response_sender, pattern);
                    } else {
                        self.list_active_channels(response_sender);
                    }
                }
                WorkerMessage::Numsub(channels, sender) => {
                    self.list_number_of_subscribers(channels, sender);
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

    /// Retorna el valor verbose del server
    ///
    /// Retorna un String "1" si verbose es true, "0" sino.
    /// Verbose true implica imprimir mensajes que describan lo que sucede en el server
    pub fn get_verbose(&self) -> &String {
        &self.verbose
    }

    /// Indica al Logger que debe imprimir un mensaje en el archivo de logs
    pub fn log(&mut self, msg: String) -> Result<(), Error> {
        self.logger.log(msg.as_bytes())?;
        Ok(())
    }

    /// Imprime un mensaje por consola
    ///
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

    /// Actualiza la lista de comandos registrados.
    ///
    /// Recibe una nueva operación de un cliente y la agrega a la lista de operaciones
    /// Busca al cliente por dirección. Si es un cliente nuevo, primero lo agrega con un OperationRegister vacio
    /// luego agrega la nueva operacion
    pub fn check_monitor(&mut self, operation: RespType, addrs: SocketAddr) {
        self.clients.iter_mut().for_each(|client| {
            if *client.is_monitoring() {
                client.write_to_stream(format!("[{}] {}", addrs, operation).as_bytes());
            }
        });
        // let last_operations = self
        //     .clients_operations
        //     .entry(addrs.to_string())
        //     .or_insert_with(|| OperationRegister::new(100));
        // last_operations.store_operation(operation);
    }

    // Escribe sobre el stream cliente todas las operaciones hechas al servidor
    // pub fn print_last_operations_by_client(&self, mut stream: TcpStream) {
    //     self.clients_operations.iter().for_each(|client| {
    //         client.1.get_operations().iter().for_each(|operation| {
    //             let op = format!("[{}] {}", client.0, operation);
    //             stream.write_all(op.as_bytes()).unwrap();
    //             stream.flush().unwrap();
    //         })
    //     });
    // }
    fn set_client_to_monitor_state(&mut self, addrs: SocketAddr) {
        self.clients.iter_mut().for_each(|client| {
            if client.get_address() == &addrs {
                //stream Ok
                client.set_monitoring(true);
            }
        });
    }

    /// Suscribe un cliente al channel
    ///
    /// Primero chequea si el channel ya existe, si existe agrega al cliente
    /// Sino lo crea y agrega al cliente y su sender
    pub fn subscribe_to_channel(
        &mut self,
        channel: String,
        addrs: SocketAddr,
        sender: Sender<usize>,
        stream: TcpStream,
    ) {
        let tx = sender.clone();

        if let Some(subscribers) = self.channels.get_mut(&channel) {
            subscribers.insert(addrs.to_string(), (sender, stream));
        } else {
            let mut inner_map = HashMap::new();
            inner_map.insert(addrs.to_string(), (sender, stream));
            self.channels.entry(channel).or_insert(inner_map);
        }
        let listening_channels = &self.get_listening_channels(addrs);

        tx.send(*listening_channels).unwrap();
        self.update_client_subscribe_status(addrs, true);
    }

    /// Actualiza el estado de suscripcion de un cliente
    fn update_client_subscribe_status(&mut self, addrs: SocketAddr, status: bool) {
        self.clients.iter_mut().for_each(|client| {
            if client.get_address() == &addrs {
                client.set_subscribe(status);
            }
        });
    }

    /// Retorna la cantidad de canales a los que esta suscrito el cliente
    fn get_listening_channels(&self, addrs: SocketAddr) -> usize {
        let mut listening_channels = 0;
        self.channels.iter().for_each(|channel| {
            if channel.1.get(&addrs.to_string()).is_some() {
                listening_channels += 1;
            }
        });
        listening_channels
    }

    /// Desuscribe al cliente de todos los canales a los que este suscrito
    ///
    /// Por el sender asociado envia mensaje para dejar de aceptar
    pub fn unsubscribe_to_all_channels(&mut self, addrs: SocketAddr, sender: Sender<usize>) {
        let mut removed = false;
        for subscriber in self.channels.values_mut() {
            match subscriber.get(&addrs.to_string()) {
                Some(_) => {
                    subscriber.remove(&addrs.to_string());
                    removed = true;
                }
                None => break,
            }
        }
        let listening_channels = self.get_listening_channels(addrs);
        sender.send(listening_channels).unwrap();
        if removed {
            self.update_client_subscribe_status(addrs, false);
        }
    }

    /// Desuscribe al cliente del canal especificado
    ///
    /// Elimina la dirección del hashmap de suscriptores de dicho canal
    pub fn unsubscribe(&mut self, channel: String, addrs: SocketAddr, tx: Sender<usize>) {
        let subscribers = self.channels.get_mut(&channel).unwrap();
        if subscribers.get(&addrs.to_string()).is_some() {
            subscribers.remove(&addrs.to_string());
            let listening_channels = self.get_listening_channels(addrs);
            tx.send(listening_channels).unwrap();
            self.update_client_subscribe_status(addrs, false);
        }
    }

    /// Envia un mensaje a todas los clientes suscritos al canal especificado
    pub fn send_message_to_channel(&mut self, channel: String, msg: String) -> usize {
        let mut sent = 0;
        let subscribers = self.channels.get_mut(&channel).unwrap();
        for sender in subscribers.values_mut() {
            sender
                .1
                .write_all(
                    parser_service::parse_response(RespType::RArray(vec![
                        RespType::RBulkString(String::from("message")),
                        RespType::RBulkString(channel.clone()),
                        RespType::RBulkString(msg.clone()),
                    ]))
                    .as_bytes(),
                )
                .unwrap();
            sender.1.flush().unwrap();
            sent += 1
        }
        sent
    }

    /// Envia al cliente una lista de todos los canales activos
    fn list_active_channels(&self, sender: Sender<Vec<RespType>>) {
        let mut channels = Vec::new();
        self.channels.iter().for_each(|channel| {
            if !channel.1.is_empty() {
                channels.push(RespType::RBulkString(channel.0.to_string()));
            }
        });
        sender.send(channels).unwrap();
    }

    /// Envia al cliente una lista de todos los canales activos que sigan el patrón especificado
    fn list_active_channels_by_pattern(&self, sender: Sender<Vec<RespType>>, pattern: String) {
        let mut channels = Vec::new();

        self.channels.iter().for_each(|channel| {
            if !channel.1.is_empty()
                && glob_pattern::g_match(pattern.as_bytes(), channel.0.as_bytes())
            {
                channels.push(RespType::RBulkString(channel.0.to_string()));
            }
        });
        sender.send(channels).unwrap();
    }

    /// Envia al cliente una lista con la cantidad de suscriptores por canal
    fn list_number_of_subscribers(&self, channels: Vec<String>, sender: Sender<Vec<RespType>>) {
        let mut list = Vec::new();
        channels.iter().for_each(|channel| {
            let mut counter = 0;
            if let Some(subscribers) = self.channels.get(channel) {
                counter = subscribers.len();
            }
            list.push(RespType::RBulkString(channel.to_string()));
            list.push(RespType::RBulkString(counter.to_string()));
        });
        sender.send(list).unwrap();
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
