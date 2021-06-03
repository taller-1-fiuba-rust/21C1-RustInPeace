use super::utils::resp_type::RespType;
use crate::domain::implementations::operation_register_impl::OperationRegister;
use crate::services::commands::command_server;
use std::collections::HashMap;
use std::net::SocketAddr;

pub struct Commander {
    operations: HashMap<String, OperationRegister>,
}

impl Commander {
    pub fn new() -> Self {
        let operations = HashMap::new();
        Commander { operations }
    }

    pub fn handle_command(&mut self, operation: &RespType, addrs: SocketAddr) {
        let last_operations = self
            .operations
            .entry(addrs.to_string())
            .or_insert_with(|| OperationRegister::new(100));
        last_operations.store_operation(operation);
        if let RespType::RArray(array) = operation {
            if let RespType::RBulkString(part_command) = &array[0] {
                match part_command.as_str() {
                    "monitor" => match self.operations.get(&addrs.to_string()) {
                        Some(operations) => {
                            let last_ops = operations.get_operations();
                            command_server::monitor(last_ops);
                        }
                        None => println!("Client doesnt exist"),
                    },
                    "info" => println!("completar.."),
                    _ => {}
                }
            }
        }
    }
    /*
    pub fn handle_command(self, command: &RespType, addrs: SocketAddr) -> Result<RespType, Error> {
        if let RespType::RArray(array) = command {
            if let RespType::RBulkString(part_command) = &array[0] {
                match part_command.as_str() {
                    "monitor" => {
                        match self.operations.get(&addrs.to_string()) {
                            Some(operations) => {
                                let last_ops = operations.get_operations();
                                command_server::monitor(last_ops);
                            }
                            None => println!("Client doesnt exist")
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(RespType::RNullArray())
    }*/
}

#[test]
fn test_01_se_guarda_una_operacion_de_tipo_info_en_operation_register() {
    use std::net::{IpAddr, Ipv4Addr};

    let mut commander = Commander::new();
    let dummy_operation = &RespType::RArray(vec![RespType::RBulkString(String::from("info"))]);
    let mut operation_register = OperationRegister::new(100);
    operation_register.store_operation(dummy_operation);

    let dir = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    commander.handle_command(dummy_operation, dir);
    println!("{:?}", commander.operations);
    let saved_operations = commander.operations.get(&dir.to_string()).unwrap();
    assert_eq!(
        saved_operations.get_operations(),
        operation_register.get_operations()
    );
}

#[test]
fn test_02_se_guardan_multiples_operaciones_en_register_operation() {
    use std::net::{IpAddr, Ipv4Addr};

    let mut commander = Commander::new();
    let dummy_operation = &RespType::RArray(vec![RespType::RBulkString(String::from("info"))]);
    let dummy_operation_2 = &RespType::RArray(vec![
        RespType::RBulkString(String::from("set")),
        RespType::RBulkString(String::from("key")),
        RespType::RBulkString(String::from("value")),
    ]);

    let mut operation_register = OperationRegister::new(100);
    operation_register.store_operation(dummy_operation);
    operation_register.store_operation(dummy_operation_2);

    let dir = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    commander.handle_command(dummy_operation, dir);
    commander.handle_command(dummy_operation_2, dir);

    println!("{:?}", commander.operations);
    let saved_operations = commander.operations.get(&dir.to_string()).unwrap();
    assert_eq!(
        saved_operations.get_operations(),
        operation_register.get_operations()
    );
}
