//! Punto de entrada de la aplicación.

use crate::domain::entities::config::Config;
use crate::domain::entities::server::Server;
use crate::domain::implementations::database::Database;
use crate::services;
use crate::services::parser_service;
use crate::services::utils::resp_type::RespType;
use crate::services::web_server_parser_service;

use std::env::args;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

static SERVER_ARGS: usize = 2;

/// Inicia el servidor Redis
///
/// Toma un argumento de la linea de comandos con la dirección de la configuración
/// y la utiliza para iniciar el server y cargar la base de datos en memoria.
/// El servidor es iniciado en la dirección "127.0.0.1".
/// El archivo de configuración debe tener definidos los siguientes campos: verbose, port, timeout, dbfilename, logfile.
/// De faltar algun parámetro de configuración, se corta la ejecución del programa.
pub fn run_redis_server() {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != SERVER_ARGS {
        println!("Cantidad de argumentos inválida");
    }

    let path = &argv[1];
    let config_path = path.clone();
    let config = Config::new(String::from(path));
    let dbfilename = config
        .get_attribute(String::from("dbfilename"))
        .expect("Error: Database config not set.");
    let dir = String::from("127.0.0.1");
    let logfile = config
        .get_attribute(String::from("logfile"))
        .expect("Error: Log config not set.");
    let verbose = config
        .get_attribute(String::from("verbose"))
        .expect("Error: Verbose config not set.");
    let db = Database::new(dbfilename);
    let (server_sender, server_receiver) = mpsc::channel();
    let server_receiver = Arc::new(Mutex::new(server_receiver));
    let port = config
        .get_attribute(String::from("port"))
        .expect("Error: Port config not set.");
    let t = thread::spawn(|| {
        let mut server = Server::new(port, logfile, verbose, server_receiver, config_path)
            .expect("Server couldn't be created.");
        server.listen();
    });
    services::server_service::init(db, config, dir, server_sender);
    t.join()
        .unwrap_or_else(|_| println!("Couldn't join server thread"));
}

/// Inicia el servidor web
///
///
pub fn run_web_server() {
    let t = thread::spawn(|| {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        let red_stream = TcpStream::connect("127.0.0.1:7001").unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let redis_stream = red_stream.try_clone().unwrap();
            handle_connection(stream, redis_stream);
        }
    });

    // tendria que pasar lo siguiente cuando se cierra la pestaña para que borre el log
    let default_contents = fs::read_to_string("redis_default.html").unwrap();
    fs::write("./redis.html", default_contents).unwrap();
}

fn handle_connection(mut stream: TcpStream, mut redis_stream: TcpStream) {
    let mut buffer = [0; 1024];
    let mut buffer_redis = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let mut contents = fs::read_to_string("redis.html").unwrap();
    let post = b"POST / HTTP/1.1\r\n";
    let commands_pos = contents.find("id=\"command-input\"").unwrap();
    let start_pos = commands_pos - 17;
    if buffer.starts_with(post) {
        let parsed_cmd = web_server_parser_service::parse_request(&buffer[..]);
        let tmp = format!("{:?}", parsed_cmd); //HACERLO BIEN implementando display
        let redis_cmd = parsed_cmd.iter().map(|e| RespType::RBulkString(e.to_string())).collect();
        let html_cmd = format!("\r\n<div class=\"command-input\">\r\n<span>></span>\r\n<span>{}</span>\r\n</div>\r\n", tmp);
        contents.insert_str(start_pos, &html_cmd);
        fs::write("./redis.html", &contents).unwrap(); //sobreescribo el html asi no pierdo los comandos anteriores (otra opcion es guardarlo en memoria)
        let parsed_cmd = parser_service::parse_response(RespType::RArray(redis_cmd));
        println!("parsed: {}", parsed_cmd);
        
        redis_stream.write(parsed_cmd.as_bytes()).unwrap();
        let size = redis_stream.read(&mut buffer_redis).unwrap();
        println!("respuesta: {:?}", &buffer_redis[..size]);
        let commands_pos = contents.find("id=\"command-input\"").unwrap();
        let start_pos = commands_pos - 17;
        let html_res = format!("\r\n<div class=\"command-response\">\r\n<p>{}</p>\r\n</div>\r\n", String::from_utf8_lossy(&buffer_redis[..size]));
        contents.insert_str(start_pos, &html_res);
        fs::write("./redis.html", &contents).unwrap(); //sobreescribo el html asi no pierdo los comandos anteriores
    }

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}