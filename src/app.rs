use crate::domain::entities::config::Config;
use crate::domain::entities::server::Server;
use crate::domain::implementations::database::Database;
use crate::services;

use std::env::args;

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
    let logfile = config.get_attribute(String::from("logfile")).unwrap();
    let verbose = config.get_attribute(String::from("verbose")).unwrap();
    let db = Database::new(dbfilename);
    match &mut Server::new(port, logfile, verbose) {
        Ok(server) => {
            services::server_service::init(server, db, config);
        }
        Err(e) => {
            println!("Error al crear el server");
            println!("Mensaje de error: {:?}", e);
        }
    }
}
