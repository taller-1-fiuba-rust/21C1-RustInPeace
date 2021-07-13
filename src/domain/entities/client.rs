use std::net::{SocketAddr, TcpStream};

#[derive(Debug)]
pub struct Client {
    addrs: SocketAddr,
    stream: TcpStream,
    subscriber: bool,
}

impl Client {
    pub fn new(addrs: SocketAddr, stream: TcpStream) -> Self {
        let subscriber = false;
        Client {
            addrs,
            stream,
            subscriber,
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
}
