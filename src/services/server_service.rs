use super::parser_service::{parse_request, parse_response};
use super::worker_service::ThreadPool;
use crate::domain::entities::client::Client;
use crate::domain::entities::config::Config;
use crate::domain::entities::message::WorkerMessage;
use crate::domain::implementations::database::Database;
use crate::services::commander::handle_command;
use crate::services::database_service::dump_to_file;
use crate::services::utils::resp_type::RespType;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::thread;

// macro_rules! select {
//     (
//         $($name:pat = $rx:ident.$meth:ident() => $code:expr),+
//     ) => ({
//         use $crate::sync::mpsc::Select;
//         let sel = Select::new();
//         $( let mut $rx = sel.handle(&$rx); )+
//         unsafe {
//             $( $rx.add(); )+
//         }
//         let ret = sel.wait();
//         $( if ret == $rx.id() { let $name = $rx.$meth(); $code } else )+
//         { unreachable!() }
//     })
// }

/// Inicia la conexion TCP
///
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
    let database = Arc::new(RwLock::new(db));
    let conf = Arc::new(RwLock::new(config));
    let cloned_db = database.clone();

    match TcpListener::bind(format!("{}:{}", dir, port)) {
        Ok(listener) => {
            // Creo un thread para que vaya iterando mientras el server esté up
            thread::spawn(move || {
                dump_to_file(cloned_db);
            });
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let tx = server_sender.clone();
                        let conf_lock = conf.clone();
                        let cloned_database = database.clone();
                        pool.spawn(|| {
                            handle_connection(stream, tx, cloned_database, conf_lock);
                        });
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

/// Lee e interpreta mensajes del cliente.
///
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
    // stop: Sender<bool>,
    // drop: Arc<AtomicBool>
) {
    let client_addrs = stream.peer_addr().unwrap();
    let client = Client::new(client_addrs, stream.try_clone().unwrap());
    tx.send(WorkerMessage::AddClient(client)).unwrap();

    log(
        format!("Connection to address {} established\r\n", client_addrs),
        &tx,
    );

    // stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
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

                match parse_request(&buf[..size]) {
                    Ok(parsed_request) => {
                        log(format!("Parsed request: {:?}\r\n", parsed_request), &tx);

                        tx.send(WorkerMessage::NewOperation(
                            parsed_request.clone(),
                            client_addrs,
                        ))
                        .unwrap();
                        println!("{:?}", parsed_request);
                        // if check_shutdown(&parsed_request) {
                        //     // drop.store(true, Ordering::Relaxed);
                        //     // println!("{:?}", drop.load(Ordering::Relaxed));
                        //     stop.send(true).unwrap();
                        //     tx.send(WorkerMessage::Stop(true)).unwrap();
                        //     break;
                        // }

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
                // stop.send(false).unwrap();
            }
            Err(e) => {
                println!("Closing connection: {:?}", e);
                break;
            }
        }
    }

    tx.send(WorkerMessage::CloseClient(client_addrs)).unwrap();
    log(
        format!("Connection to address {} closed\r\n", client_addrs),
        &tx,
    );
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
fn _check_shutdown(request: &RespType) -> bool {
    if let RespType::RArray(array) = request {
        if let RespType::RBulkString(cmd) = &array[0] {
            if cmd == "shutdown" {
                return true;
            }
        }
    }
    false
}
