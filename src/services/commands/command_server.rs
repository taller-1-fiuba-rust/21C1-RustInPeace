use crate::entities::resp_types::RespType;

pub fn monitor(operations: &[Vec<String>]) {
    for operation in operations {
        println!("{:?}", operation)
    }
}

pub fn info(cmd: &Vec<RespType>) -> RespType {
    let mut option = "default".to_string();
    if cmd.len() == 2 {
        if let RespType::RBulkString(comando) = &cmd[1] {
            option = comando.to_string();
        }
    }

    match option.as_str() {
        "server" => {
            RespType::RBulkString("# Server\r\nredis_version:6.2.3\r\nredis_git_sha1:00000000\r\nredis_git_dirty:0\r\nredis_build_id:ea3be5cbc55dfd19\r\n".to_string())
        }
        "clients" => {
            RespType::RBulkString("# Clients\r\nconnected_clients:2\r\ncluster_connections:0\r\nmaxclients:10000\r\n".to_string())
        }
        "persistence" => RespType::RNullArray(),
        // "stats" => {}
        // "replication" => {}
        // "cpu" => {}
        // "commandstats" => {}
        // "cluster" => {}
        // "modules" => {}
        // "keyspace" => {}
        // "errorstats" => {}
        // "all" => {}
        // "default" => {}
        // "everything" => {}
        _ => RespType::RNullArray(),
    }
}
