//!# Redis Oxidado
//!## <ins>Proyecto:</ins> Taller de Programación I
//!## Facultad de Ingeniería de la Universidad de Buenos Aires
//!---
//!### Equipo: ###
//!
//!Lionel Guglielmone  
//!<ins>Padrón</ins>: 96963
//!
//!Daniela Leloutre  
//!<ins>Padrón</ins>: 96783
//!
//!María Sol Orive  
//!<ins>Padrón</ins>: 91351
//!
//!
//![Repositorio](https://github.com/taller-1-fiuba-rust/RustInPeace)  
//![Consigna](https://taller-1-fiuba-rust.github.io/proyecto/Proyecto_2021_1C_Redis.pdf)
//! ## Introducción ##
//! El proyecto corresponde a un servidor que se comporta como una base de datos <clave, valor> al estilo Redis.
//!
//!## Uso ##
//!### Instalación ###
//!Requisitos para correr localmente el servidor:
//!1. [Rust](https://www.rust-lang.org/es/tools/install)
//!2. Cargo (Si instalaste Rust con el link anterior, cargo viene incluído)
//!
//!Una vez instaladas las herramientas, ejecutar:  
//!```ignore
//!cargo run <redis.conf>
//!```
//!donde <redis.conf> es la ubicación del archivo de configuración del servidor.
//!
//!### Configuración ###
//!
//!El archivo de configuración del servidor por default se encuentra en ```src/redis.conf```
//!Los valores posibles a modificar son:
//!```ignore
//!verbose 1
//!port 8080
//!timeout 300
//!dbfilename dump.rdb
//!logfile /var/log/redis/redis-server.log
//!```
//!
//!## Persistencia ##
//!
//!Los datos almacenados en el servidor se bajan a un archivo *dump* definido en el archivo de configuración del servidor.
//!La bajada de datos se realiza de manera periódica cada 2 minutos, eliminando todo el archivo *dump* y haciendo una bajada completa con los datos almacenados en memoria en ese instante.
//!
//!Si la bajada no completase, se imprime un mensaje en el archivo log y se continúa con la ejecución.
//!
//!### Tipos de keys ###
//!
//!Existen 2 tipos de keys en el servidor: las volátiles y las persistentes.
//!Las claves volátiles son aquellas que tiene un TTL asociado, es decir, aquellas par alas cuales el cliente puede setearles un tiempo de expiración (el servidor se encargará de removerlas de manera automática). Por otro lado, las de tipo persistente son keys que vivirán dentro del server hasta que el cliente decida eliminarlas.
//!
//!En el archivo *dump* se localiza la información del tipo de clave en la tercera columna. Por ejemmplo, se puede encontrar una clave persistente de esta manera:
//!```ignore
//!mykey;1626111469;;string;Hello
//!```
//!Para el caso de una clave de tipo volátil se agregará su tiempo de expiración. Ejemplo: para *mykey* se le agrega un timestamp de expiración *1635597186*
//!```ignore
//!mykey;1626111469;1635597186;string;Hello
//!```
//!
//!### Formato key-value ###
//!Cada tupla key-value se persiste en el archivo *dump* de la siguiente manera:
//!```ignore
//!<key>;<last_access_time>;<timeout>;<value_type>;<values>
//!```
//!donde:  
//!```<key>```: Identificador único para la key   
//!```<last_access_time>```: Timestamp con el último acceso a la key  
//!```<timeout>```:  Timestamp con el tiempo de expiración de la key  
//!```<value_type>```: Tipo de dato almacenado (Set, String o List)   
//!```<values>```:   Valores de la key  
//!
//! ### Detalles de implementación ###
//! Se modeló el proyecto siguiendo el enfoque Domain-Driven Design (DDD) con el propósito de acercar la implementación al dominio y encapsular
//! la lógica e implementación en otro lugar.
//!
//! Para el diseño presentado, se divididio la estructura en tres capas: de servicios, de implementación y finalmente otra de base de datos.
//! El objetivo de esta idea es que el cliente ingrese solicitando un servicio, entonces que este servicio delegue el pedido a cierta implementación y que esta última tenga acceso
//! a la base de datos. Asi de esta forma se aislan los datos del cliente.
//!
//! ![alt text](../../../src/images/diagrama3.jpeg "Diseño del proyecto.")
//!
//!
//! El servicio `server_service` es quien recibe conexiones y lee solicitudes que luego delega a un servicio llamado `commander`; `commander` se encarga
//! de interpretar el tipo de solicitud para definir a qué otro servicio le corresponde procesarla. Los servicios disponibles son: command_string, command_key,
//! command_server, command_set, command_pubsub y command_list. Dependiendo de el tipo de solicitud, dichos se decide sobre si se deberá (o no) acceder a la base de datos.
//!
//!
//! ![alt text](../../../src/images/diagrama2.jpeg "Servicios de comandos y sus funciones")
//!
//!
//!
//! Como las solicitudes llegan siguiendo el protocolo RESP de Redis, se implemento un servicio denominado `parser_service` para leer esas solicitudes y transformarlas a un tipo de dato conveniente.
//! Se definio para ello el enum `RespType` con los valores: RSimpleString, RBulkString, RError, RNullBulkstrin, RArray, RNullArray, RInteger y RSignedNumber.
//! Tanto las solicitudes como las respuestas a las mismas, deben pasar por este servicio.
//!
//! ![alt text](../../../src/images/diagrama1.jpeg "Diagrama de componentes.")
//!
//! Al iniciar la aplicación, se crea un canal que sirve para comunicar a cada cliente con el hilo de ejecución principal.
//! En un hilo de ejecución nuevo se instancia la entidad `Server`. Este `Server` tiene en su poder el "receiver" del canal previamente mencionado; mientras dure la ejecución del programa va a quedar pendiente de mensajes que le puedan llegar del hilo principal o de clientes.
//! Los mensajes que puede recibir el servidor son de tipo `WorkerMessage`:
//! * Log: escribe un mensaje en el archivo log, cuya direccion está definida en el archivo de config.
//! * Verb: imprime mensajes por consola, indicando el funcionamiento interno del servidor.
//! * NewOperation: registra el ultimo comando ingresado por el cliente.
//! * MonitorOp: devuelve todas las operaciones registradas por el servidor.
//! * Subscribe: suscribe al cliente a un canal dado.
//! * Unsubscribe: desuscribe al cliente del canal dado.
//! * UnsubscribeAll: desuscribe al cliente de todos los canales a los que se haya suscrito.
//! * Publish: publica un mensaje en los canales especificados.
//! * Channels: lista canales activos.
//! * Numsub: lista cantidad de suscriptores por canal.
//!
//! El `Server` contiene información de los clientes conectados, los canales de suscripción, los comandos procesados, entre otros.
//! Cada vez que se conecta un nuevo cliente, se le otorga al mismo una copia del sender del canal que se comunica con el servidor.
//! En aquellos casos en los que el cliente requiera una respuesta del servidor, se crea un nuevo canal y se le envía el sender de este último dentro del WorkerMessage de
//! modo tal que el servidor pueda enviar por allí la respuesta. Este comportamiento se puede ver en funciones como Monitor e Info.
//!
//! ![alt text](../../../src/images/diagrama6.jpeg "Comunicación entre clientes y servidor.")
//!
//! En el servicio `server_service` se crea la conexión TCP y se reciben clientes de forma concurrente. Las conexiones son enviadas a un Threadpool (instanciado en `worker_service`),
//! y cada nueva conexión (o cliente) le es asignada a un `Worker` (un thread) que va a encargarse de procesar sus solicitudes.
//! En este contexto, un `Worker` es un hilo de ejecución distinto.
//!
//! ![alt text](../../../src/images/diagrama7.jpeg "Comunicación entre clientes y servidor.")
//!

pub mod app;
pub mod domain;
pub mod errors;
pub mod services;
