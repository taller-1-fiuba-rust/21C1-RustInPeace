mod domain;
mod errors;
mod repositories;
mod services;

use domain::entities::config::Config;
use domain::entities::key_value_item::{KeyValueItem, ValueType};
use domain::entities::server::Server;
use domain::implementations::database::Database;
use std::collections::{HashSet, LinkedList};
use std::env::args;

static SERVER_ARGS: usize = 2;

fn main() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        println!("Cantidad de argumentos inválida");
        // return Err(());
    }

    let path = &argv[1];
    let config = Config::new(String::from(path));
    let dbfilename = config.get_attribute(String::from("dbfilename")).unwrap();
    let port = config.get_attribute(String::from("port")).unwrap();
    let logfile = config.get_attribute(String::from("logfile")).unwrap();
    let verbose = config.get_attribute(String::from("verbose")).unwrap();
    let mut db = Database::new(dbfilename.to_string());
    match &mut Server::new(port.to_string(), logfile.to_string(), verbose.to_string()) {
        Ok(server) => {
            services::server_service::init(server, config);
            //pongo estos prints para que no tire warning de funciones sin usar (las uso en los tests)
            println!("{}", db.get_filename());
            println!("{}", db.get_size());
            let kvi = KeyValueItem::new("1".to_string(), ValueType::StringType("hola".to_string()));

            let kvi2 = KeyValueItem::new("1".to_string(), ValueType::ListType(LinkedList::new()));
            let kvi3 = KeyValueItem::new("1".to_string(), ValueType::SetType(HashSet::new()));
            db.add(kvi);
            db.add(kvi2);
            db.add(kvi3);

            db.delete_by_index(0);
            db.delete_by_index(1);
            db.delete_by_index(2);
            // fin del codigo a borrar
        }
        Err(e) => {
            println!("Error al crear el server");
            println!("Mensaje de error: {:?}", e);
        }
    }
}
