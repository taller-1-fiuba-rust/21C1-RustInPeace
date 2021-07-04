use super::parser_service::{parse_request, parse_response};
use super::worker_service::ThreadPool;
use crate::domain::entities::client::Client;
use crate::domain::entities::config::Config;
use crate::domain::entities::message::WorkerMessage;
// use crate::domain::entities::server::Server;
use crate::domain::implementations::database::Database;
use crate::services::commander::handle_command;
// use crate::services::parser_service;
use crate::services::utils::resp_type::RespType;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};

// static drop: AtomicBool = AtomicBool::new(false);

/// Recibe una refencia mutable de tipo Server, la base de datos Database y la configuración Config
/// Crea un Threadpool con X workers (definir) y en un hilo de ejecución distinto crea una conexión TCP
/// que va a escuchar mensajes hasta que se le envíe una señal de "shutdown".
pub fn init(
    db: Database,
    config: Config,
    port: String,
    dir: String,
    server_sender: Sender<WorkerMessage>,
) {
    let pool = ThreadPool::new(4);
    // let drop = Arc::new(AtomicBool::new(false));

    let database = Arc::new(RwLock::new(db));
    let conf = Arc::new(RwLock::new(config));
    let (stop_signal_sender, stop_signal_receiver) = mpsc::channel();
    
    match TcpListener::bind(format!("{}:{}", dir, port)) {
        Ok(listener) => {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let tx = server_sender.clone();
                        let conf_lock = conf.clone();
                        let cloned_database = database.clone();
                        let stop = stop_signal_sender.clone();
                        pool.spawn(|| {
                            handle_connection(stream, tx, cloned_database, conf_lock, stop);
                        });
                    
                        if let Ok(drop) = stop_signal_receiver.recv() {
                            if drop {
                                println!("DROP");
                                save_database(database);
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        println!("Couldn't get stream");
                        continue;
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

/// Recibe una base de datos de tipo Database protegida por un RwLock
/// y guarda la información en su correspondiente archivo
fn save_database(database: Arc<RwLock<Database>>) {
    println!("Saving dump before shutting down");
    let x = Arc::try_unwrap(database);
    match x {
        Ok(t) => {
            match t.try_read() {
                Ok(n) => n.save_items_to_file(),
                Err(_) => unreachable!(),
            };
        }
        Err(_) => {
            println!("Database couldn't be saved into file");
        }
    }
}

/// Recibe un stream proveniente de la conexión TCP, un sender de mensajes de tipo WorkerMessage, una base de datos de tipo Database dentro de un RwLock
/// la configuración config dentro de un RwLock y un sender de mensajes de tipo booleano stop.
/// Lee el stream de datos recibido del cliente, lo decodifica, mediante la función handle_command realiza la operación que corresponda y luego
/// escribe una respuesta sobre el mismo stream. La lectura se hace dentro de un ciclo loop hasta recibir la señal de "stop" por parte del cliente
/// o hasta que se cierre la conexión por parte del cliente o se produzca algún error interno.
pub fn handle_connection(
    mut stream: TcpStream,
    tx: Sender<WorkerMessage>,
    database: Arc<RwLock<Database>>,
    config: Arc<RwLock<Config>>,
    stop: Sender<bool>,
    // drop: Arc<AtomicBool>
) {
    let client_addrs = stream.peer_addr().unwrap();
    let client = Client::new(client_addrs, stream.try_clone().unwrap());
    log(
        format!("Connection to address {} established\r\n", client_addrs),
        &tx,
    );

    // stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
    loop {
        let mut buf = [0u8; 512];
        match stream.read(&mut buf) {
            Ok(0) => {
                println!("CERO");
                break;
            }
            Ok(size) => {
                log(
                    format!(
                        "Reading new message from {}. Message: {:?}\r\n",
                        client_addrs,
                        String::from_utf8_lossy(&buf[..size])
                    ),
                    &tx,
                );
                // stop.send(false).unwrap();
                match parse_request(&buf[..size]) {
                    Ok(parsed_request) => {
                        log(format!("Parsed request: {:?}\r\n", parsed_request), &tx);

                        tx.send(WorkerMessage::NewOperation(
                            parsed_request.clone(),
                            client_addrs,
                        ))
                        .unwrap();
                        println!("{:?}", parsed_request);
                        if check_shutdown(&parsed_request) {
                            println!("soy shutdown");
                            // drop.store(true, Ordering::Relaxed);
                            // println!("{:?}", drop.load(Ordering::Relaxed));
                            stop.send(true).unwrap();
                            tx.send(WorkerMessage::Stop(true)).unwrap();
                            println!("break server service");
                            return;
                        }
                        // stop.send(false).unwrap();
                        if let Some(res) = handle_command(
                            parsed_request,
                            &tx,
                            client_addrs,
                            &database,
                            &config,
                            stream.try_clone().unwrap(),
                        ) {
                            let response = parse_response(res);
                            log(
                                format!(
                                    "Response for {}. Message: {:?}. Response: {}\r\n",
                                    client_addrs,
                                    String::from_utf8_lossy(&buf[..size]),
                                    response
                                ),
                                &tx,
                            );
                            stream.write_all(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                        }
                    }
                    Err(e) => {
                        println!("Error trying to parse request: {:?}", e);
                        continue;
                    }
                }
                stop.send(false).unwrap();
            }
            Err(e) => {
                println!("Closing connection: {:?}", e);
                break;
            }
        }
    }
    // stop.send(false).unwrap();
    println!("salgo de handle_connection");
}

/// Recibe un mensaje msg de tipo String y un sender tx de mensajes de tipo WorkerMessage
/// El sender envia el mensaje Log
fn log(msg: String, tx: &Sender<WorkerMessage>) {
    tx.send(WorkerMessage::Log(msg)).unwrap();
}

/// Recibe un mensaje msg de tipo String y un sender tx de mensajes de tipo WorkerMessage
/// El sender envia el mensaje Verbose
fn _verbose(msg: String, tx: &Sender<WorkerMessage>) {
    tx.send(WorkerMessage::Verb(msg)).unwrap();
}

/// Recibe una solicitud request de tipo &RespType y valida si es el comando "SHUTDOWN"
/// Devuelve true si lo es, false si no
fn check_shutdown(request: &RespType) -> bool {
    if let RespType::RArray(array) = request {
        if let RespType::RBulkString(cmd) = &array[0] {
            if cmd == "shutdown" {
                return true;
            }
        }
    }
    false
}
