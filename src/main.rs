mod entities;
mod services;

use crate::entities::resp_types::RespType;
use entities::config::Config;
use entities::operation_register::OperationRegister;
use entities::server::Server;
use services::command_service;

fn main() {
    //por aca habria que crear un obj. config y pasarselo al server
    //(o sea en vez de recibir un puerto, recibe el config)
    let config = Config::new();
    // //let port: String = "8080".to_string();
    let server = Server::new(config);
    services::server_service::init(server);
    //----------------------------------
    let mut register = OperationRegister::new();
    //se crea el RespType que contiene un comando al servidor
    let elemento_1 = RespType::RBulkString("set".to_string());
    let elemento_2 = RespType::RBulkString("key".to_string());
    let elemento_3 = RespType::RBulkString("value".to_string());
    let vector_aux = vec![elemento_1, elemento_2, elemento_3];
    let operation = RespType::RArray(vector_aux);

    command_service::register_operation(&mut register, operation);
    register.monitor();
    //las variables algo_i las creo para que clippy no se queje
    let algo_1 = RespType::RError("no hay error".to_string()); //RBulkString("set_b".to_string());
    let algo_2 = RespType::RNullBulkString();
    let algo_3 = RespType::RNullArray();
    let algo_4 = RespType::RInteger(2);
    let algo_5 = RespType::RSimpleString("corto".to_string());
    //los print debajo son para ue clippy no se queje
    println!("{:?}", algo_1);
    println!("{:?}", algo_2);
    println!("{:?}", algo_3);
    println!("{:?}", algo_4);
    println!("{:?}", algo_5);
}

//#[test]
