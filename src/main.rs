mod entities;
mod services;

use entities::config::Config;
use entities::server::Server;

fn main() {
    //por aca habria que crear un obj. config y pasarselo al server
    //(o sea en vez de recibir un puerto, recibe el config)
    let config = Config::new();
    //let port: String = "8080".to_string();
    let server = Server::new(config);
    services::server_service::init(server);
}
