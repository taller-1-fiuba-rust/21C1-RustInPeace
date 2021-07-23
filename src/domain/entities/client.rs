//! Representación de un cliente del Servidor.

use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
};

/// Representa un cliente conectado al servidor.
/// Se compone por su dirección de origen, el stream sobre el cual se leen comandos y escriben respuestas, un estado de suscripción y un estado de monitoreo.
/// La dirección de origen es única y se compone por una IP y un puerto.
/// Un cliente pasa a estar en estado "suscrito" cuando ejecuta el comando `subscribe`, permanece en dicho estado hasta que se desuscriba con el comando `unsubscribe`.
/// Un cliente pasa a estar en estado "monitor" cuando ejecuta el comando `monitor`, permanece en dicho estado hasta que detenga la conexión con ctrl-c.
/// Ambos estados son bloqueantes, es decir que el cliente no podrá enviar otro comando mientras se encuentre en alguno de ellos.
#[derive(Debug)]
pub struct Client {
    addrs: SocketAddr,
    stream: TcpStream,
    subscriber: bool,
    monitoring: bool,
}

impl Client {
    /// Crea una instancia del cliente.
    ///
    /// Inicia al cliente con los estados `subscriber` y `monitoring` en false.
    pub fn new(addrs: SocketAddr, stream: TcpStream) -> Self {
        let subscriber = false;
        let monitoring = false;
        Client {
            addrs,
            stream,
            subscriber,
            monitoring,
        }
    }

    /// Retorna una referencia al stream del cliente.
    pub fn get_stream(&self) -> &TcpStream {
        &self.stream
    }

    /// Retorna una referencia mutable al stream del cliente.
    pub fn get_stream_mut(&mut self) -> TcpStream {
        self.stream.try_clone().unwrap()
    }

    /// Retorna una referencia a la dirección del cliente.
    pub fn get_address(&self) -> &SocketAddr {
        &self.addrs
    }

    /// Retorna si el cliente se encuentra en estado "monitoring".
    pub fn is_subscriber(&self) -> &bool {
        &self.subscriber
    }

    /// Actualiza el estado "subscribe".
    pub fn set_subscribe(&mut self, subs: bool) {
        self.subscriber = subs;
    }

    /// Retorna si el cliente se encuentra en estado "monitoring".
    pub fn is_monitoring(&self) -> &bool {
        &self.monitoring
    }

    /// Actualiza el estado "monitoring".
    pub fn set_monitoring(&mut self, monitor: bool) {
        self.monitoring = monitor;
    }

    /// Escribe un mensaje sobre el stream del cliente.
    pub fn write_to_stream(&mut self, message: &[u8]) {
        self.stream.write_all(message).unwrap();
    }
}
