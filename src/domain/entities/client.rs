use std::{io::Write, net::{SocketAddr, TcpStream}};

#[derive(Debug)]
pub struct Client {
    addrs: SocketAddr,
    stream: TcpStream,
    subscriber: bool,
    monitoring: bool,
}

impl Client {
    pub fn new(addrs: SocketAddr, stream: TcpStream) -> Self {
        let subscriber = false;
        let monitoring = false;
        Client {
            addrs,
            stream,
            subscriber,
            monitoring
        }
    }

    pub fn get_stream(&self) -> &TcpStream {
        &self.stream
    }

    pub fn get_stream_mut(&mut self) -> TcpStream {
        self.stream.try_clone().unwrap()
    }

    pub fn get_address(&self) -> &SocketAddr {
        &self.addrs
    }

    pub fn is_subscriber(&self) -> &bool {
        &self.subscriber
    }

    pub fn set_subscribe(&mut self, subs: bool) {
        self.subscriber = subs;
    }

    pub fn is_monitoring(&self) -> &bool {
        &self.monitoring
    }

    pub fn set_monitoring(&mut self, monitor: bool) {
        self.monitoring = monitor;
    }

    pub fn write_to_stream(&mut self, message: &[u8]) {
        self.stream.write_all(message).unwrap();
    }
}
