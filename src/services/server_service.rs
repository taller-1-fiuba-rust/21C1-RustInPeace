//! Servicio para iniciar el servidor y manejar mensajes de clientes.

use super::parser_service::{parse_request, parse_response};
use super::worker_service::ThreadPool;
use crate::domain::entities::client::Client;
use crate::domain::entities::config::Config;
use crate::domain::entities::message::WorkerMessage;
use crate::domain::implementations::database::Database;
use crate::services::commander::handle_command;
use crate::services::database_service::dump_to_file;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{thread, u64};

/// Inicia la conexion TCP
///
/// Crea un Threadpool con 10 workers y en un hilo de ejecución distinto crea una conexión TCP
/// que va a quedar pendiente de recibir clientes y mensajes nuevos.
/// Establece un channel entre la entidad `Server` y el cliente para que cada cliente pueda recibir y enviar información
/// al servidor de manera concurrente.
/// En un tercer hilo de ejecución se hace una bajada periódica de los datos almacenados en Database al archivo `dump.rdb`.
/// Si la configuración no tiene especificado un timeout válido, se asigna 300 segundos por defecto.
pub fn init(db: Database, config: Config, dir: String, server_sender: Sender<WorkerMessage>) {
    let port = config
        .get_attribute(String::from("port"))
        .expect("Error: Port config not set.");
    let timeout = config
        .get_attribute("timeout".to_string())
        .expect("Error: Timeout config not set.")
        .parse::<u64>()
        .unwrap_or(300);
    let pool = ThreadPool::new(10);
    let database = Arc::new(RwLock::new(db));
    let conf = Arc::new(RwLock::new(config));
    let cloned_db = database.clone();

    match TcpListener::bind(format!("{}:{}", dir, port)) {
        Ok(listener) => {
            thread::spawn(move || {
                dump_to_file(cloned_db);
            });
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let tx = server_sender.clone();
                        let conf_lock = conf.clone();
                        let cloned_database = database.clone();
                        stream
                            .set_read_timeout(Some(Duration::from_secs(timeout)))
                            .expect("Could not set a read timeout");

                        pool.spawn(|| {
                            handle_connection(stream, tx, cloned_database, conf_lock).unwrap_or(());
                        });
                    }
                    Err(_) => {
                        println!("Couldn't get stream");
                        continue;
                    }
                }
            }
        }
        Err(e) => {
            panic!("Listener couldn't be created. Error: {}", e.to_string());
        }
    }
    println!("Shutting down...");
}

/// Lee e interpreta mensajes del cliente.
///
/// Recibe un stream proveniente de la conexión TCP, un sender de mensajes de tipo WorkerMessage, una base de datos de tipo Database dentro de un RwLock
/// y la configuración Config dentro de un RwLock.
/// Lee el stream de datos recibido del cliente, lo decodifica, mediante la función handle_command realiza la operación que corresponda y luego
/// escribe una respuesta sobre el mismo stream. La lectura se hace dentro de un ciclo loop
/// hasta que se cierre la conexión por parte del cliente o se produzca algún error interno.
pub fn handle_connection(
    mut stream: TcpStream,
    tx: Sender<WorkerMessage>,
    database: Arc<RwLock<Database>>,
    config: Arc<RwLock<Config>>,
) -> Result<(), Box<dyn Error>> {
    let client_addrs = stream.peer_addr()?;
    let client = Client::new(client_addrs, stream.try_clone()?);
    tx.send(WorkerMessage::AddClient(client))
        .expect("Could not send client to server");

    log(
        format!("Connection to address {} established\r\n", client_addrs),
        &tx,
    );
    verbose(
        format!("Connection to address {} established\r\n", client_addrs),
        &tx,
    );

    loop {
        let mut buf = [0u8; 512];
        match stream.read(&mut buf) {
            Ok(0) => {
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
                verbose(
                    format!(
                        "Reading new message from {}. Message: {:?}\r\n",
                        client_addrs,
                        String::from_utf8_lossy(&buf[..size])
                    ),
                    &tx,
                );

                match parse_request(&buf[..size]) {
                    Ok(parsed_request) => {
                        log(format!("Parsed request: {:?}\r\n", parsed_request), &tx);
                        verbose(format!("Parsed request: {:?}\r\n", parsed_request), &tx);
                        let mut subscribed = false;
                        let (ps_sender, ps_recv) = mpsc::channel();
                        tx.send(WorkerMessage::NewOperation(
                            parsed_request.clone(),
                            client_addrs,
                            ps_sender,
                        ))
                        .unwrap();

                        if let Ok(pubsub_state) = ps_recv.recv() {
                            subscribed = pubsub_state;
                        }

                        let res = handle_command(
                            parsed_request,
                            &tx,
                            client_addrs,
                            &database,
                            &config,
                            subscribed,
                        )?;
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
                        verbose(
                            format!(
                                "Response for {}. Message: {:?}. Response: {}\r\n",
                                client_addrs,
                                String::from_utf8_lossy(&buf[..size]),
                                response
                            ),
                            &tx,
                        );
                        stream.write_all(response.as_bytes())?;
                        stream.flush()?;
                    }
                    Err(e) => {
                        println!("Error trying to parse request: {:?}", e);
                        continue;
                    }
                }
            }
            Err(e) => {
                println!("Closing connection: {:?}", e);
                break;
            }
        }
    }

    tx.send(WorkerMessage::CloseClient(client_addrs))
        .expect("Could not close client");
    log(
        format!("Connection to address {} closed\r\n", client_addrs),
        &tx,
    );
    verbose(
        format!("Connection to address {} closed\r\n", client_addrs),
        &tx,
    );
    Ok(())
}

/// Envia un mensaje al Logger.
///
/// El sender envia el mensaje al servidor para que lo escriba en el archivo de logs.
fn log(msg: String, tx: &Sender<WorkerMessage>) {
    tx.send(WorkerMessage::Log(msg))
        .expect("Could not send log.");
}

/// Imprime un mensaje por consola.
///
/// El sender envia el mensaje al servidor para que lo imprima.
fn verbose(msg: String, tx: &Sender<WorkerMessage>) {
    tx.send(WorkerMessage::Verb(msg))
        .expect("Could not send verbose");
}
