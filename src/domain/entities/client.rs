use std::net::{SocketAddr, TcpStream};

#[derive(Debug)]
pub struct Client {
    addrs: SocketAddr,
    stream: TcpStream
}

impl Client {
    pub fn new(addrs: SocketAddr, stream: TcpStream) -> Self {
        Client { addrs, stream }
    }

    pub fn get_stream(&self) -> &TcpStream {
        &self.stream
    }

    pub fn get_stream_mut(&mut self) -> TcpStream {
        self.stream.try_clone().unwrap()
    }

    pub fn get_address(self) -> SocketAddr {
        self.addrs
    }
}