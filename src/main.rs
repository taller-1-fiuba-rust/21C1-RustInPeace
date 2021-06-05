mod domain;
mod errors;
mod repositories;
mod services;

use domain::entities::server::Server;
use services::config_service::load_config;

fn main() {
    let path = "./src/redis.conf".to_string();
    let config = load_config(path);
    match config {
        Ok(conf) => match &mut Server::new(conf) {
            Ok(server) => {
                services::server_service::init(server);
            }
            Err(e) => {
                println!("Error al crear el server");
                println!("Mensaje de error: {:?}", e);
            }
        },
        Err(_) => {
            println!(
                "No se pudo cargar la configuracion. Se establece una configuracion por default"
            )
        }
    };
}
