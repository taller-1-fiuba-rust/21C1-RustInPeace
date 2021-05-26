use super::commands::server;
use std::io::Error;

pub fn call_command(input: Vec<String>) -> Result<Vec<String>, Error> {
    let mut op_result = Vec::<String>::new();
    match input[0].as_str() {
        "info" => {
            if input.len() == 1 {
                op_result = server::info("default".to_string());
            } else {
                op_result = server::info(input[1].to_string());
            }
            println!("{:?}", op_result); //esto se devuelve, no se imprime
        }
        _ => {
            println!("No");
        }
    };
    Ok(op_result)
    // "monitor" => {},
    // "flushdb" => {},
    // "config get" => {},
    // "config set" => {},
    // "dbsize" => {},
    // "copy" => {},
    // "del" => {},
    // "exists" => {},
    // "expire" => {},
    // "expireat" => {},
    // "keys" => {},
    // "persist" => {},
    // "rename" => {},
    // "sort" => {},
    // "touch" => {},
    // "ttl" => {},
    // "type" => {},
    // "append" => {},
    // "decrby" => {},
    // "get" => {},
    // "getdel" => {},
    // "getset" => {},
    // "incrby" => {},
    // "mget" => {},
    // "mset" => {},
    // "set" => {},
    // "strlen" => {},
    // "lindex" => {},
    // "llen" => {},
    // "lpop" => {},
    // "lpush" => {},
    // "lpushx" => {},
    // "lrange" => {},
    // "lrem" => {},
    // "lset" => {},
    // "rpop" => {},
    // "rpush" => {},
    // "rpushx" => {},
    // "sadd" => {},
    // "scard" => {},
    // "sismember" => {},
    // "smembers" => {},
    // "srem" => {},
    // "pubsub" => {},
    // "publish" => {},
    // "subscribe" => {},
    // "unsubcribe" => {},
}
