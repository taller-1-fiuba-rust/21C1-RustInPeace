//! Punto de entrada de la aplicación.

use crate::domain::entities::config::Config;
use crate::domain::entities::server::Server;
use crate::domain::implementations::database::Database;
use crate::services;
use crate::services::parser_service;
use crate::services::web_server_parser_service;
use crate::services::worker_service::ThreadPool;

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

/// Inicia el servidor web.
///
/// Inicia el servidor web en la dirección "127.0.0.1:8080" y
/// se conecta al servidor Redis en el puerto 7001.
pub fn run_web_server() {
    thread::spawn(|| {
        let pool = ThreadPool::new(10);
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        let default_contents = fs::read_to_string("redis_default.html").unwrap(); //esto deberia pasar al final o al abrir pestaña nueva
        fs::write("./redis.html", default_contents).unwrap();
        let redis_stream = TcpStream::connect("127.0.0.1:7001").unwrap(); //clonado, verificar que no se haya cerrado
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let redis = redis_stream.try_clone().unwrap();
            pool.spawn(|| {
                handle_connection(stream, redis);
                println!("cerro la conexion");
                // let default_contents = fs::read_to_string("redis_default.html").unwrap(); //esto deberia pasar al final o al abrir pestaña nueva
                // fs::write("./redis.html", default_contents).unwrap();
            });
        }
    });
}

/// Resuelve peticiones del servidor web.
///
/// Si la solicitud es de tipo POST, obtiene el comando Redis enviado en la solicitud, se la envía al servidor Redis e imprime el comando en el archivo HTML asociado al servidor web.
/// Luego, recibe del servidor Redis una respuesta que también imprime sobre el archivo HTML para poder mostrarle al cliente del servidor web.
fn handle_connection(mut stream: TcpStream, mut redis_stream: TcpStream) {
    let mut contents = fs::read_to_string("redis.html").expect("Could not read HTML file.");
    let mut buffer = [0; 1024];
    let mut buffer_redis = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!(
                    "{} Web Server - Closed connection",
                    stream.peer_addr().unwrap()
                );
                break;
            }
            Ok(size) => {
                println!("{} Llega mensaje", stream.peer_addr().unwrap());
                let post = b"POST / HTTP/1.1\r\n";
                if buffer.starts_with(post) {
                    let parsed_cmd = web_server_parser_service::parse_request(&buffer[..size]);
                    let html_cmd = format!(
                        "\r\n<div class=\"command-input\">\r\n<span>></span>\r\n<span>{}</span>\r\n</div>\r\n",
                        web_server_parser_service::get_body_as_string(&parsed_cmd)
                    );
                    // overwrite_file(
                    //     "redis.html",
                    //     html_cmd,
                    //     &mut contents,
                    //     17,
                    //     "id=\"command-input\"",
                    // );
                    let pos = contents.find("id=\"command-input\"").unwrap() - 17;
                    contents.insert_str(pos, &html_cmd);

                    let redis_cmd = web_server_parser_service::get_body_as_resp(parsed_cmd);
                    let parsed_cmd = parser_service::parse_response(redis_cmd);
                    // envio comando redis al servidor
                    redis_stream.write_all(parsed_cmd.as_bytes()).unwrap();
                    // leo respuesta del servidor redis
                    match redis_stream.read(&mut buffer_redis) {
                        Ok(0) => {
                            println!("Redis Server - Closed connection");
                        }
                        Ok(size) => {
                            let html_res = format!(
                                "\r\n<div class=\"command-response\">\r\n<p>{}</p>\r\n</div>\r\n",
                                String::from_utf8_lossy(&buffer_redis[..size])
                            );
                            let pos = contents.find("id=\"command-input\"").unwrap() - 17;
                            contents.insert_str(pos, &html_res);
                            // overwrite_file(
                            //     "redis.html",
                            //     html_res,
                            //     &mut contents,
                            //     17,
                            //     "id=\"command-input\"",
                            // );
                        }
                        Err(_) => {
                            println!("Could not read redis response.");
                        }
                    }
                }
                println!("writing http response");
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", //revisar
                    contents.len(),
                    contents
                );

                stream.write_all(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            Err(_) => {
                println!("Could not read web server stream");
                break;
            }
        }
        // println!("writing http response");
        // let response = format!(
        //     "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}\r\n", //revisar
        //     contents.len(),
        //     contents
        // );

        // stream.write_all(response.as_bytes()).unwrap();
        // stream.flush().unwrap();
    }
}

/// Sobreescribe un archivo con su contenido modificado.
///
/// Busca el substring `search` dentro de `contents` y a la posición de inicio del substring le resta `offset`. La posición resultante es
/// la posición donde se va a insertar el substring `new_contents`.
/// Actualiza el archivo `filename` con el contenido final.
pub fn overwrite_file(
    filename: &str,
    new_contents: String,
    contents: &mut String,
    offset: usize,
    search: &str,
) {
    let pos = contents.find(search).unwrap() - offset;
    contents.insert_str(pos, &new_contents);
    fs::write(filename, contents).unwrap(); //sobreescribo el html asi no pierdo los comandos anteriores (otra opcion es guardarlo en memoria)
}
