//! Servidor Redis

use super::client::Client;
use super::message::WorkerMessage;
use crate::domain::implementations::logger_impl::Logger;
use crate::services::parser_service;
use crate::services::utils::glob_pattern;
use crate::services::utils::resp_type::RespType;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::process;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::usize;
use std::{io::Error, net::SocketAddr};

/// Servidor que procesa comandos de clientes Redis.
///
/// El servidor se compone por los siguientes elementos:
/// * dir: dirección donde recibe conexiones.
/// * port: puerto donde recibe conexiones.
/// * verbose: indica si debe imprimir mensajes sobre su comportamiento.
/// * logger: estructura que escribe mensajes sobre su comportamiento en un archivo log.
/// * clients: vector de clientes conectados.
/// * total_connections: cantidad total de conexiones establecidas.
/// * total_commands: cantidad total de comandos procesados.
/// * channels: lista de canales establecidos, diferenciados por el nombre de canal.
/// * receiver: receptor de mensajes provenientes del cliente.
/// * init_time: almacena el tiempo en que fue iniciado el servidor.
/// * config_path: dirección del archivo de configuración.
#[derive(Debug)]
pub struct Server {
    dir: String,
    port: String,
    verbose: String,
    logger: Logger,
    clients: Vec<Client>,
    total_connections: usize,
    total_commands: usize,
    channels: HashMap<String, HashMap<String, (Sender<usize>, TcpStream)>>,
    receiver: Arc<Mutex<mpsc::Receiver<WorkerMessage>>>,
    init_time: SystemTime,
    config_path: String,
}

impl Server {
    /// Crea una nueva instancia Server.
    ///
    /// Por defecto, inicia el servidor en la dirección 127.0.0.1
    /// # Ejemplo
    /// ```
    /// use proyecto_taller_1::domain::entities::server::Server;
    /// use std::sync::{Arc, Mutex, mpsc};
    ///
    /// let port = "8080".to_string();
    /// let logfile = "./src/dummy_logfile.out".to_string();
    /// let verbose = "1".to_string();
    /// let (sender, receiver) = mpsc::channel();
    /// let recv = Arc::new(Mutex::new(receiver));
    /// let configfile = "./src/dummy_configfile.txt".to_string();
    ///
    /// let server = Server::new(port, logfile, verbose, recv, configfile);
    /// # let _ = std::fs::remove_file("./src/dummy_logfile.out");
    /// ```
    pub fn new(
        port: String,
        logfile: String,
        verb: String,
        receiver: Arc<Mutex<Receiver<WorkerMessage>>>,
        config_path: String,
    ) -> Result<Self, Error> {
        let dir = "127.0.0.1".to_string();
        let port = port;
        let verbose = verb;
        let receiver = receiver;
        let logger_path = &logfile;
        let logger = Logger::new(logger_path)?;
        let channels = HashMap::new();
        let clients = Vec::new();
        let init_time = SystemTime::now();
        let config_path = config_path;
        let total_connections = 0;
        let total_commands = 0;

        Ok(Server {
            dir,
            port,
            verbose,
            logger,
            clients,
            total_connections,
            total_commands,
            channels,
            receiver,
            init_time,
            config_path,
        })
    }

    /// Escucha mensajes provenientes de los workers, según el mensaje delega al server una tarea distinta.
    ///
    /// Esta es una función bloqueante, ya que el servidor va a estar pendiente de mensajes de sus clientes
    /// en tanto esté ejecutandose el hilo principal.
    ///
    /// Las tareas pueden ser:
    /// * Log: escribe un mensaje en el archivo log, cuya direccion está definida en el archivo de config.
    /// * Verb: imprime mensajes por consola, indicando el funcionamiento interno del servidor.
    /// * NewOperation: registra el ultimo comando ingresado por el cliente.
    /// * MonitorOp: devuelve todas las operaciones registradas por el servidor.
    /// * Subscribe: suscribe al cliente a un canal dado.
    /// * Unsubscribe: desuscribe al cliente del canal dado.
    /// * UnsubscribeAll: desuscribe al cliente de todos los canales a los que se haya suscrito.
    /// * Publish: publica un mensaje en los canales especificados.
    /// * Channels: lista canales activos.
    /// * Numsub: lista cantidad de suscriptores por canal.
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::entities::server::Server;
    /// # use std::sync::{Arc, Mutex, mpsc};
    /// use std::thread;
    ///
    /// # let port = "8080".to_string();
    /// # let logfile = "./src/dummy_logfile.out".to_string();
    /// # let verbose = "1".to_string();
    /// # let (sender, receiver) = mpsc::channel();
    /// # let recv = Arc::new(Mutex::new(receiver));
    /// # let configfile = "./src/dummy_configfile.txt".to_string();
    ///
    /// let mut server = Server::new(port, logfile, verbose, recv, configfile).unwrap();
    /// thread::spawn(move || server.listen());
    /// # let _ = std::fs::remove_file("./src/dummy_logfile.out");
    /// ```
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
                    self.set_client_to_monitor_state(addrs);
                }
                WorkerMessage::AddClient(client) => {
                    self.clients.push(client);
                    self.total_connections += 1;
                }
                WorkerMessage::CloseClient(addrs) => {
                    self.remove_client(addrs);
                }
                WorkerMessage::NewOperation(operation, addrs) => {
                    self.check_monitor(operation, addrs);
                    self.total_commands += 1;
                }
                WorkerMessage::InfoServer(sender) => {
                    sender.send(self.get_server_info()).unwrap();
                }
                WorkerMessage::InfoClients(sender) => {
                    sender.send(self.get_clients_info()).unwrap();
                }
                WorkerMessage::InfoStats(sender) => {
                    sender.send(self.get_stats_info()).unwrap();
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

    /// Retorna el puerto donde escucha el server.
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::entities::server::Server;
    /// # use std::sync::{Arc, Mutex, mpsc};
    ///
    /// # let port = "8080".to_string();
    /// # let logfile = "./src/dummy_logfile.out".to_string();
    /// # let verbose = "1".to_string();
    /// # let (sender, receiver) = mpsc::channel();
    /// # let recv = Arc::new(Mutex::new(receiver));
    /// # let configfile = "./src/dummy_configfile.txt".to_string();
    ///
    /// let mut server = Server::new(port, logfile, verbose, recv, configfile).unwrap();
    /// assert_eq!(server.get_port(), &String::from("8080"));
    /// # let _ = std::fs::remove_file("./src/dummy_logfile.out");
    /// ```
    pub fn get_port(&self) -> &String {
        &self.port
    }

    /// Retorna la dirección IP del server.
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::entities::server::Server;
    /// # use std::sync::{Arc, Mutex, mpsc};
    ///
    /// # let port = "8080".to_string();
    /// # let logfile = "./src/dummy_logfile.out".to_string();
    /// # let verbose = "1".to_string();
    /// # let (sender, receiver) = mpsc::channel();
    /// # let recv = Arc::new(Mutex::new(receiver));
    /// # let configfile = "./src/dummy_configfile.txt".to_string();
    ///
    /// let mut server = Server::new(port, logfile, verbose, recv, configfile).unwrap();
    /// assert_eq!(server.get_dir(), &String::from("127.0.0.1"));
    /// # let _ = std::fs::remove_file("./src/dummy_logfile.out");
    /// ```
    pub fn get_dir(&self) -> &String {
        &self.dir
    }

    /// Retorna el valor verbose del server.
    ///
    /// Retorna un String "1" si verbose es true, "0" sino.
    /// Verbose true implica imprimir mensajes que describan lo que sucede en el servidor.
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::entities::server::Server;
    /// # use std::sync::{Arc, Mutex, mpsc};
    ///
    /// # let port = "8080".to_string();
    /// # let logfile = "./src/dummy_logfile.out".to_string();
    /// # let verbose = "1".to_string();
    /// # let (sender, receiver) = mpsc::channel();
    /// # let recv = Arc::new(Mutex::new(receiver));
    /// # let configfile = "./src/dummy_configfile.txt".to_string();
    ///
    /// let mut server = Server::new(port, logfile, verbose, recv, configfile).unwrap();
    /// assert_eq!(server.get_verbose(), &String::from("1"));
    /// # let _ = std::fs::remove_file("./src/dummy_logfile.out");
    /// ```
    pub fn get_verbose(&self) -> &String {
        &self.verbose
    }

    /// Indica al Logger que debe imprimir un mensaje en el archivo de logs.
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::entities::server::Server;
    /// # use std::sync::{Arc, Mutex, mpsc};
    ///
    /// # let port = "8080".to_string();
    /// # let logfile = "./src/dummy_logfile.out".to_string();
    /// # let verbose = "1".to_string();
    /// # let (sender, receiver) = mpsc::channel();
    /// # let recv = Arc::new(Mutex::new(receiver));
    /// # let configfile = "./src/dummy_configfile.txt".to_string();
    ///
    /// let mut server = Server::new(port, logfile, verbose, recv, configfile).unwrap();
    /// assert!(server.log(String::from("random log message")).is_ok());
    /// # let _ = std::fs::remove_file("./src/dummy_logfile.out");
    /// ```
    pub fn log(&mut self, msg: String) -> Result<(), Error> {
        self.logger.log(msg.as_bytes())?;
        Ok(())
    }

    /// Imprime un mensaje por consola.
    ///
    /// Si verbose es 1 (true), imprime el mensaje recibido.
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::entities::server::Server;
    /// # use std::sync::{Arc, Mutex, mpsc};
    ///
    /// # let port = "8080".to_string();
    /// # let logfile = "./src/dummy_logfile.out".to_string();
    /// # let verbose = "1".to_string();
    /// # let (sender, receiver) = mpsc::channel();
    /// # let recv = Arc::new(Mutex::new(receiver));
    /// # let configfile = "./src/dummy_configfile.txt".to_string();
    ///
    /// let mut server = Server::new(port, logfile, verbose, recv, configfile).unwrap();
    /// server.verbose(String::from("random verbose message"));
    /// # let _ = std::fs::remove_file("./src/dummy_logfile.out");
    /// ```
    pub fn verbose(&self, msg: String) {
        if self.parse_verbose(self.get_verbose()) == 1 {
            println!("{}", msg);
        }
    }

    /// Devuelve información del servidor Redis.
    ///
    /// Retorna un string con la siguiente información:
    /// * redis_version: Version del servidor Redis
    /// * redis_git_sha1: Git SHA1
    /// * redis_git_dirty: Git dirty flag
    /// * redis_build_id: Build id
    /// * redis_mode: Modo del servidor ("standalone", "sentinel" o "cluster")
    /// * os: Sistema operativo sobre el que corre el servidor Redis
    /// * arch_bits: Arquitectura (32 o 64 bits)
    /// * multiplexing_api: Atomicvar API utilizada por Redis
    /// * gcc_version: Version del compilador GCC
    /// * process_id: PID del proceso del servidor
    /// * run_id: Valor random para identificar al servidor Redis
    /// * tcp_port: Puerto de escucha TCP/IP
    /// * server_time_in_usec: Tiempo del sistema basado en EPOCH con presicion de microsegundos
    /// * uptime_in_seconds: Segundos desde que se inició el servidor Redis
    /// * uptime_in_days: Días desde que se inició el servidor Redis
    /// * hz: Frecuencia actual del servidor
    /// * configured_hz: Frecuencia configurada
    /// * lru_clock: Reloj que se incrementa cada minuto
    /// * executable: Dirección del archivo ejecutable del servidor
    /// * config_file: Dirección del archivo de configuración
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::entities::server::Server;
    /// # use std::sync::{Arc, Mutex, mpsc};
    ///
    /// # let port = "8080".to_string();
    /// # let logfile = "./src/dummy_logfile.out".to_string();
    /// # let verbose = "1".to_string();
    /// # let (sender, receiver) = mpsc::channel();
    /// # let recv = Arc::new(Mutex::new(receiver));
    /// # let configfile = "./src/dummy_configfile.txt".to_string();
    ///
    /// let mut server = Server::new(port, logfile, verbose, recv, configfile).unwrap();
    /// server.get_server_info();
    /// # let _ = std::fs::remove_file("./src/dummy_logfile.out");
    /// ```
    pub fn get_server_info(&self) -> String {
        let info = format!("# Server\r\nredis_version:6.2.3\r\nredis_git_sha1:00000000\r\nredis_git_dirty:0\r\nredis_build_id:ea3be5cbc55dfd19\r\nredis_mode:standalone\r\nos:Linux 5.4.0-1030-aws x86_64\r\narch_bits:64\r\nmultiplexing_api:epoll\r\natomicvar_api:c11-builtin\r\ngcc_version:9.3.0\r\nprocess_id:{}\r\nprocess_supervised:no\r\nrun_id:eba2478b32af796180fdf364700b411432cb6932\r\ntcp_port:{}\r\nserver_time_usec:{}\r\nuptime_in_seconds:{}\r\nuptime_in_days:{}\r\nhz:10\r\nconfigured_hz:10\r\nlru_clock:15868238\r\nexecutable:/usr/local/bin/redis-server\r\nconfig_file:{}\r\n", process::id(), self.get_port(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros(), SystemTime::now().duration_since(self.init_time).unwrap().as_secs(), SystemTime::now().duration_since(self.init_time).unwrap().as_secs()/86400, self.config_path);
        info
    }

    /// Devuelve información de los clientes conectados al servidor Redis.
    ///
    /// Retorna un string con la siguiente información:
    /// * connected_clients: Cantidad de clientes conectados
    /// * cluster_connections: Una aproximación del número de sockets utilizados por el bus del clúster
    /// * maxclients: Máxima cantidad de clientes
    /// * client_longest_output_list: Lista de salida más larga entre las conexiones de clientes actuales
    /// * client_biggest_input_buf: El búfer de entrada más grande entre las conexiones de clientes actuales
    /// * blocked_clients: Cantidad de clientes pendientes en una llamada bloqueante
    /// * tracking_clients: Cantidad de clientes siendo rastreados
    /// * clients_in_timeout_table: Cantidad de clientes en la tabla de timeout
    /// * io_threads_active: Flag que indica si hay threads de I/O activos
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::entities::server::Server;
    /// # use std::sync::{Arc, Mutex, mpsc};
    ///
    /// # let port = "8080".to_string();
    /// # let logfile = "./src/dummy_logfile.out".to_string();
    /// # let verbose = "1".to_string();
    /// # let (sender, receiver) = mpsc::channel();
    /// # let recv = Arc::new(Mutex::new(receiver));
    /// # let configfile = "./src/dummy_configfile.txt".to_string();
    ///
    /// let mut server = Server::new(port, logfile, verbose, recv, configfile).unwrap();
    /// assert_eq!(server.get_clients_info(), String::from("# Clients\r\nconnected_clients:0\r\ncluster_connections:0\r\nmaxclients:10000\r\nclient_longest_output_list:24\r\nclient_biggest_input_buf:50\r\nblocked_clients:0\r\ntracking_clients:0\r\nclients_in_timeout_table:0\r\nio_threads_active:0\r\n"));
    /// # let _ = std::fs::remove_file("./src/dummy_logfile.out");
    /// ```
    pub fn get_clients_info(&self) -> String {
        let info = format!("# Clients\r\nconnected_clients:{}\r\ncluster_connections:0\r\nmaxclients:10000\r\nclient_longest_output_list:24\r\nclient_biggest_input_buf:50\r\nblocked_clients:0\r\ntracking_clients:0\r\nclients_in_timeout_table:0\r\nio_threads_active:0\r\n", self.clients.len());
        info
    }

    /// Devuelve estadísticas del servidor Redis.
    ///
    /// Retorna un string con la siguiente información:
    /// * total_connections_received: Cantidad de conexiones establecidas
    /// * total_commands_processed: Cantidad de comandos procesados
    /// * total_net_input_bytes: Número de bytes leídos
    /// * total_net_output_bytes: Número de bytes escritos
    /// * rejected_connections: Cantidad de conexiones rechazadas
    /// * expired_keys: Cantidad de claves expiradas
    /// * keyspace_hits: Cantidad de búsquedas de claves exitosas
    /// * keyspace_misses: Cantidad de búsquedas de claves fallidas
    /// * pubsub_channels: Cantidad de canales pub/sub con suscripciones
    /// * total_error_replies: Cantidad total de errores emitidos como respuesta
    /// * total_reads_processed: Cantidad de lecturas procesadas
    /// * total_writes_processed: Cantidad de escrituras procesadas
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::entities::server::Server;
    /// # use std::sync::{Arc, Mutex, mpsc};
    ///
    /// # let port = "8080".to_string();
    /// # let logfile = "./src/dummy_logfile.out".to_string();
    /// # let verbose = "1".to_string();
    /// # let (sender, receiver) = mpsc::channel();
    /// # let recv = Arc::new(Mutex::new(receiver));
    /// # let configfile = "./src/dummy_configfile.txt".to_string();
    ///
    /// let mut server = Server::new(port, logfile, verbose, recv, configfile).unwrap();
    /// assert_eq!(server.get_stats_info(), String::from("# Stats\r\ntotal_connections_received:0\r\ntotal_commands_processed:0\r\ntotal_net_input_bytes:6656\r\ntotal_net_output_bytes:8192\r\nrejected_connections:0\r\nexpired_keys:0\r\nkeyspace_hits:3\r\nkeyspace_misses:2\r\npubsub_channels:0\r\ntotal_error_replies:2\r\ntotal_reads_processed:10\r\ntotal_writes_processed:5\r\n"));
    /// # let _ = std::fs::remove_file("./src/dummy_logfile.out");
    /// ```
    pub fn get_stats_info(&self) -> String {
        let info = format!("# Stats\r\ntotal_connections_received:{}\r\ntotal_commands_processed:{}\r\ntotal_net_input_bytes:6656\r\ntotal_net_output_bytes:8192\r\nrejected_connections:0\r\nexpired_keys:0\r\nkeyspace_hits:3\r\nkeyspace_misses:2\r\npubsub_channels:{}\r\ntotal_error_replies:2\r\ntotal_reads_processed:10\r\ntotal_writes_processed:5\r\n", self.total_connections, self.total_commands, self.channels.len());
        info
    }

    /// Convierte el verbose original de tipo String a tipo usize.
    ///
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::entities::server::Server;
    /// # use std::sync::{Arc, Mutex, mpsc};
    ///
    /// # let port = "8080".to_string();
    /// # let logfile = "./src/dummy_logfile.out".to_string();
    /// # let verbose = "1".to_string();
    /// # let (sender, receiver) = mpsc::channel();
    /// # let recv = Arc::new(Mutex::new(receiver));
    /// # let configfile = "./src/dummy_configfile.txt".to_string();
    ///
    /// let mut server = Server::new(port, logfile, verbose, recv, configfile).unwrap();
    /// assert_eq!(server.parse_verbose("1"), 1);
    /// # let _ = std::fs::remove_file("./src/dummy_logfile.out");
    /// ```
    pub fn parse_verbose(&self, string: &str) -> usize {
        let mut verbose: usize = 1;
        let verb_aux = string.parse::<usize>();
        match verb_aux {
            Ok(verb) => verbose = verb,
            Err(_) => println!("parsing error"),
        }
        verbose
    }

    /// Retiene todos los clientes cuya direccion sea distinta a la que se quiere eliminar.
    fn remove_client(&mut self, addrs: SocketAddr) {
        self.clients.retain(|client| client.get_address() != &addrs);
    }

    /// Envia el ultimo comando recibido a los clientes que esten en estado "monitor".
    ///
    /// Verifica si hay algun cliente monitoreando los comandos enviados al servidor.
    /// Si lo hay, le envia el ultimo comando ejecutado.
    pub fn check_monitor(&mut self, operation: RespType, addrs: SocketAddr) {
        self.clients.iter_mut().for_each(|client| {
            if *client.is_monitoring() {
                let msg = parser_service::parse_response(RespType::RBulkString(format!(
                    "[{}] {}",
                    addrs, operation
                )));
                client.write_to_stream(msg.as_bytes());
            }
        });
    }

    /// Cambia el estado de un cliente a "monitor".
    ///
    /// El cliente pasa a un estado de "debug" donde solo puede recibir los comandos que se ejecutan en el servidor.
    fn set_client_to_monitor_state(&mut self, addrs: SocketAddr) {
        self.clients.iter_mut().for_each(|client| {
            if client.get_address() == &addrs {
                client.write_to_stream(
                    parser_service::parse_response(RespType::RBulkString(String::from("Ok")))
                        .as_bytes(),
                );
                client.set_monitoring(true);
            }
        });
    }

    /// Suscribe un cliente al channel.
    ///
    /// Primero chequea si el channel ya existe, si existe agrega al cliente.
    /// Sino lo crea y agrega al cliente y su sender.
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

    /// Actualiza el estado de suscripcion de un cliente.
    ///
    /// Busca el cliente con dirección `addrs` y le asigna `status` al estado de suscripción.
    fn update_client_subscribe_status(&mut self, addrs: SocketAddr, status: bool) {
        self.clients.iter_mut().for_each(|client| {
            if client.get_address() == &addrs {
                client.set_subscribe(status);
            }
        });
    }

    /// Retorna la cantidad de canales a los que esta suscrito el cliente.
    fn get_listening_channels(&self, addrs: SocketAddr) -> usize {
        let mut listening_channels = 0;
        self.channels.iter().for_each(|channel| {
            if channel.1.get(&addrs.to_string()).is_some() {
                listening_channels += 1;
            }
        });
        listening_channels
    }

    /// Desuscribe al cliente de todos los canales a los que este suscrito.
    ///
    /// Por el sender asociado envia mensaje para dejar de aceptar mensajes del canal.
    /// Luego, actualiza el estado de suscripción del cliente.
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

    /// Desuscribe al cliente del canal especificado.
    ///
    /// Elimina la dirección del hashmap de suscriptores de dicho canal.
    /// Luego, actualiza el estado de suscripción del cliente.
    pub fn unsubscribe(&mut self, channel: String, addrs: SocketAddr, tx: Sender<usize>) {
        let subscribers = self.channels.get_mut(&channel).unwrap();
        if subscribers.get(&addrs.to_string()).is_some() {
            subscribers.remove(&addrs.to_string());
            let listening_channels = self.get_listening_channels(addrs);
            tx.send(listening_channels).unwrap();
            self.update_client_subscribe_status(addrs, false);
        }
    }

    /// Envia un mensaje a todas los clientes suscritos al canal especificado.
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

    /// Envia al cliente una lista de todos los canales activos.
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
