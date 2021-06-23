use crate::domain::entities::config::Config;
use crate::domain::entities::server;
use crate::domain::entities::server::Server;
use crate::domain::implementations::database::Database;
use crate::services;

use std::env::args;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;

static SERVER_ARGS: usize = 2;

/// Toma un argumento de la linea de comandos con el path de la configuracion
/// y la utiliza para inicializar el server y cargar la base de datos en memoria
/// Imprime un mensaje de error si la creación del server falla
pub fn run() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        println!("Cantidad de argumentos inválida");
    }

    let path = &argv[1];
    let config = Config::new(String::from(path));
    let dbfilename = config.get_attribute(String::from("dbfilename")).unwrap();
    let port = config.get_attribute(String::from("port")).unwrap();
    let dir = String::from("127.0.0.1");
    let logfile = config.get_attribute(String::from("logfile")).unwrap();
    let verbose = config.get_attribute(String::from("verbose")).unwrap();
    let db = Database::new(dbfilename);
    let (server_sender, server_receiver) = mpsc::channel();
    let server_receiver = Arc::new(Mutex::new(server_receiver));
    let port_2 = port.clone();
    let t = thread::spawn(|| {
        let mut server = Server::new(port_2, logfile, verbose, server_receiver).unwrap();
        server.listen();
    });
    // match &mut Server::new(port, logfile, verbose) {
        // Ok(server) => {
            services::server_service::init(db, config, port, dir, server_sender);
        // }
        // Err(e) => {
        //     println!("Error al crear el server");
        //     println!("Mensaje de error: {:?}", e);
        // }
    // }
    t.join().unwrap();
}
