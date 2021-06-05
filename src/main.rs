mod domain;
mod errors;
mod repositories;
mod services;

use domain::entities::key_value_item::{KeyValueItem, ValueType};
use domain::entities::server::Server;
use domain::implementations::database::Database;
use services::config_service::load_config;
use std::collections::{HashSet, LinkedList};

fn main() {
    let path = "./src/redis.conf".to_string();
    let config = load_config(path);
    match config {
        Ok(conf) => {
            let mut db = Database::new(conf.get_dbfilename().to_string());
            match &mut Server::new(conf) {
                Ok(server) => {
                    services::server_service::init(server);
                    //pongo estos prints para que no tire warning de funciones sin usar (las uso en los tests)
                    println!("{}", db.get_filename());
                    println!("{}", db.get_size());
                    let kvi = KeyValueItem::new(
                        "1".to_string(),
                        ValueType::StringType("hola".to_string()),
                    );

                    let kvi2 =
                        KeyValueItem::new("1".to_string(), ValueType::ListType(LinkedList::new()));
                    let kvi3 =
                        KeyValueItem::new("1".to_string(), ValueType::SetType(HashSet::new()));
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
        Err(_) => {
            println!(
                "No se pudo cargar la configuracion. Se establece una configuracion por default"
            )
        }
    };
}
