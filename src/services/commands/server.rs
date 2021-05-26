pub fn info(option: String) -> Vec<String> {
    let mut bulk_string_reply = Vec::<String>::new();
    let vec_server = vec![
        "# Server".to_string(),
        "redis_version:6.2.3".to_string(),
        "redis_git_sha1:00000000".to_string(),
        "redis_git_dirty:0".to_string(),
        "redis_build_id:ea3be5cbc55dfd19".to_string(),
    ];
    let vec_client = vec![
        "# Clients".to_string(),
        "connected_clients:2".to_string(),
        "cluster_connections:0".to_string(),
        "maxclients:10000".to_string(),
    ];
    // lo mismo para las demas opciones. Preguntar de donde obtenemos estos datos
    //que estan almacenados en el vector

    match option.as_str() {
        "server" => {
            for info in vec_server {
                bulk_string_reply.push(encode_info(info.to_string()))
            }
        }
        "clients" => {
            for info in vec_client {
                bulk_string_reply.push(encode_info(info.to_string()))
            }
        }
        "persistence" => {}
        "stats" => {}
        "replication" => {}
        "cpu" => {}
        "commandstats" => {}
        "cluster" => {}
        "modules" => {}
        "keyspace" => {}
        "errorstats" => {}
        "all" => {}
        "default" => {}
        "everything" => {}
        _ => {
            println!("No");
        }
    };
    bulk_string_reply
}

fn encode_info(info: String) -> String {
    let enconded_text = format!("${}\r\n{}\r\n", info.len(), info);
    enconded_text
}
