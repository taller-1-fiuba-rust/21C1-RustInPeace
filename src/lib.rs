//!# Redis Oxidado
//!## <ins>Proyecto:</ins> Taller de Programación I
//!## Facultad de Ingeniería de la Universidad de Buenos Aires
//!---
//!### Equipo: ###
//!
//!Lionel Guglielmone  
//!<ins>Padrón</ins>: TBC
//!
//!Daniela Leloutre  
//!<ins>Padrón</ins>: TBC
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
//!Para poder correr localmente el servidor se necesita:
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
//!Si la bajada no se hubiera podido completar, se imprime un mensaje en el log y se continúa con la ejecución.
//!
//!### Tipos de keys ###
//!
//!Existen 2 tipos de keys en el servidor. Por un lado, las volátiles y por otro las persistentes.
//!Las claves volátiles son aquellas que tiene un TTL asociado, es decir el cliente puede setearles un tiempo de expiración y el servidor se encargará de removerlas de manera automática. Por otro lado, las de tipo persistente son keys que vivirán dentro del server hasta que el cliente decida eliminarlas.
//!
//!En el archivo *dump* se localiza la información del tipo de clave en la tercera columna. Por ejemmplo, se puede encontrar una clave persistente de esta manera:
//!```ignore
//!mykey;1626111469;;string;Hello
//!```
//!Para el caso de una clave de tipo volátil se agregará su tiempo de expiración. Ejemplo: para *mykey* le agrego un timestamp de expiración *1635597186*
//!```ignore
//!mykey;1626111469;1635597186;string;Hello
//!```
//!
//!### Formato key-value ###
//!Cada tupla key-value se persistirá de la siguiente manera en el archivo *dump*
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
pub mod app;
pub mod domain;
pub mod errors;
pub mod services;
