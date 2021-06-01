mod entities;
mod key_value_item;
mod services;

use entities::config::Config;
use entities::server::Server;
use services::config_service::load_config;

fn main() {
    let path = "./src/redis.txt".to_string();
    let config = load_config(path);

    match config {
        Ok(conf) => {
            let server = Server::new(conf);
            services::server_service::init(server);
        }
        Err(_) => {
            println!(
                "No se pudo cargar la configuracion. Se establece una configuracion por default"
            )
        }
    };
}
