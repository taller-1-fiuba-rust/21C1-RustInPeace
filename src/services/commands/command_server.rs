use std::sync::{Arc, RwLock};
use std::io::{Error, ErrorKind};

use crate::domain::entities::config::Config;
use crate::services::utils::resp_type::RespType;

pub fn _monitor(operations: &[Vec<String>]) {
    for operation in operations {
        println!("{:?}", operation)
    }
}

pub fn config_get(config: &Arc<RwLock<Config>> ,field: &RespType) -> Result<String, Error> {
    if let RespType::RBulkString(field_name) = field {
        if let Ok(read_guard) = config.read() {
            let conf = read_guard;
            let value = conf.get_attribute(String::from(field_name))?;
            return Ok(String::from(value));
        }
        Err(Error::new(ErrorKind::InvalidInput, "Field name missing"))
    } else {
        Err(Error::new(ErrorKind::InvalidInput, "Invalid request"))
    }
}

pub fn config_set(config: &Arc<RwLock<Config>>, field: &RespType, value: &RespType) -> Result<String, Error> {
    if let RespType::RBulkString(field_name) = field {
        if let RespType::RBulkString(value) = value {
            if let Ok(write_guard) = config.write() {
                let mut conf = write_guard;
                let _value = conf.set_attribute(String::from(field_name), String::from(value))?;
                return Ok(String::from("ok"));
            }
        }
        Err(Error::new(ErrorKind::InvalidInput, "Field name missing"))
    } else {
        Err(Error::new(ErrorKind::InvalidInput, "Invalid request"))
    }
}


#[test]
fn test_config_get_verbose() {
    let _parsed_command = RespType::RBulkString(String::from("verbose"));

}