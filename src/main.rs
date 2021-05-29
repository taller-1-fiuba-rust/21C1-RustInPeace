mod entities;
mod services;

use entities::config::Config;
use entities::server::Server;
use services::config_service::load_config;

fn main() {
    //por aca habria que crear un obj. config y pasarselo al server
    //(o sea en vez de recibir un puerto, recibe el config)
    let path =
        "C:/Users/Ron Heyes/UBA/TALLER DE PROGRAMACION/TP/RustInPeace/src/redis.txt".to_string();
    let config = load_config(path);
    //el Config por default
    let mut config_ready = Config::new(
        1,
        8080.to_string(),
        64,
        "dbfilename1".to_string(),
        "logfile1".to_string(),
    );
    match config {
        Ok(conf) => config_ready = conf,
        Err(_) => {
            println!(
                "No se pudo cargar la configuracion. Se establece una configuracion por default"
            )
        }
    };

    let server = Server::new(config_ready);
    services::server_service::init(server);
}

//#[test]
