use super::config::Config;

#[derive(Debug)]
pub struct Server {
    dir: String,
    port: String,
    threadpool_size: usize
}

impl Server {
    pub fn new(config: Config) -> Self {
        let dir = "127.0.0.1".to_string();
        let threadpool_size = 4;
        let port = config.get_port();
        Server { dir, port, threadpool_size }
    }

    pub fn get_port(&self) -> &String {
        &self.port
    }

    pub fn get_dir(&self) -> &String {
        &self.dir
    }

    pub fn get_threadpool_size(&self) -> &usize {
        &self.threadpool_size
    }
}