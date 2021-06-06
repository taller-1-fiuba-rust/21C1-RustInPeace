// use super::config::Config;
use crate::domain::implementations::logger_impl::Logger;
use crate::domain::implementations::operation_register_impl::OperationRegister;
use crate::services::utils::resp_type::RespType;
use std::collections::HashMap;
use std::{io::Error, net::SocketAddr};

#[derive(Debug)]
pub struct Server {
    dir: String,
    port: String,
    threadpool_size: usize,
    logger: Logger, // receiver: Arc<Mutex<mpsc::Receiver<WorkerMessage>>>
    clients_operations: HashMap<String, OperationRegister>,
}

impl Server {
    pub fn new(port: String, logfile: String) -> Result<Self, Error> {
        let dir = "127.0.0.1".to_string();
        let threadpool_size = 4;
        let port = port;
        // let receiver = receiver;
        let logger_path = &logfile;
        let logger = Logger::new(logger_path)?;
        let clients_operations = HashMap::new();

        Ok(Server {
            dir,
            port,
            threadpool_size,
            logger,
            clients_operations,
        })
    }

    pub fn get_port(&self) -> &String {
        &self.port
    }

    pub fn get_dir(&self) -> &String {
        &self.dir
    }

    pub fn get_threadpool_size(&self) -> &usize {
        &self.threadpool_size
    }

    pub fn log(&mut self, msg: String) -> Result<(), Error> {
        self.logger.log(msg.as_bytes())?;
        Ok(())
    }

    pub fn update_clients_operations(&mut self, operation: RespType, addrs: SocketAddr) {
        let last_operations = self
            .clients_operations
            .entry(addrs.to_string())
            .or_insert_with(|| OperationRegister::new(100));
        last_operations.store_operation(operation);
    }

    pub fn print_last_operations_by_client(&self, addrs: String) {
        if let Some(operations) = self.clients_operations.get(&addrs) {
            for operation in operations.get_operations() {
                println!("{:?}", operation)
            }
        }
    }
}

#[test]
fn test_01_se_guarda_una_operacion_de_tipo_info_en_operation_register() {
    // use super::config::Config;
    use super::server::Server;
    use std::net::{IpAddr, Ipv4Addr};

    // let verbose = 0;
    // let timeout = 0;
    let port = "8080".to_string();
    // let dbfilename = "./src/redis.conf".to_string();
    let logfile = "./src/dummy.log".to_string();

    let mut server = Server::new(port, logfile).unwrap();
    let dummy_operation = RespType::RArray(vec![RespType::RBulkString(String::from("info"))]);
    let mut operation_register = OperationRegister::new(100);
    operation_register.store_operation(dummy_operation.clone());

    let dir = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    server.update_clients_operations(dummy_operation, dir);
    let saved_operations = server.clients_operations.get(&dir.to_string()).unwrap();
    assert_eq!(
        saved_operations.get_operations(),
        operation_register.get_operations()
    );

    std::fs::remove_file("./src/dummy.log").unwrap();
}

#[test]
fn test_02_se_guardan_multiples_operaciones_en_register_operation() {
    // use super::config::Config;
    use super::server::Server;
    use std::net::{IpAddr, Ipv4Addr};

    // let verbose = 0;
    // let timeout = 0;
    let port = "8080".to_string();
    // let dbfilename = "./src/redis.conf".to_string();
    let logfile = "./src/dummy.log".to_string();

    let mut server = Server::new(port, logfile).unwrap();
    let dummy_operation = RespType::RArray(vec![RespType::RBulkString(String::from("info"))]);
    let dummy_operation_2 = RespType::RArray(vec![
        RespType::RBulkString(String::from("set")),
        RespType::RBulkString(String::from("key")),
        RespType::RBulkString(String::from("value")),
    ]);

    let mut operation_register = OperationRegister::new(100);
    operation_register.store_operation(dummy_operation.clone());
    operation_register.store_operation(dummy_operation_2.clone());

    let dir = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    server.update_clients_operations(dummy_operation, dir);
    server.update_clients_operations(dummy_operation_2, dir);

    let saved_operations = server.clients_operations.get(&dir.to_string()).unwrap();
    assert_eq!(
        saved_operations.get_operations(),
        operation_register.get_operations()
    );

    std::fs::remove_file("./src/dummy.log").unwrap();
}