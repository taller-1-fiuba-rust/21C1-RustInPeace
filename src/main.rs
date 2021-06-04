mod entities;
mod key_value_item;
mod services;

use crate::entities::database::Database;
use crate::key_value_item::key_value_item_domain::{KeyValueItem, ValueType};
use entities::config::Config;
use entities::server::Server;
use services::config_service::load_config;
use std::collections::{HashSet, LinkedList};

fn main() {
    let path = "./src/redis.txt".to_string();
    let config = load_config(path);

    match config {
        Ok(conf) => {
            let mut db = Database::new(conf.get_dbfilename().to_string());
            let server = Server::new(conf);

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

            services::server_service::init(server);
        }
        Err(_) => {
            println!(
                "No se pudo cargar la configuracion. Se establece una configuracion por default"
            )
        }
    };
}
