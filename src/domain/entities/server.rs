use super::config::Config;
use crate::domain::implementations::logger_impl::Logger;
use std::io::Error;

#[derive(Debug)]
pub struct Server {
    dir: String,
    port: String,
    threadpool_size: usize,
    logger: Logger, // receiver: Arc<Mutex<mpsc::Receiver<WorkerMessage>>>
}

impl Server {
    pub fn new(config: Config) -> Result<Self, Error> {
        let dir = "127.0.0.1".to_string();
        let threadpool_size = 4;
        let port = config.get_port().to_string();
        // let receiver = receiver;
        let logger_path = config.get_logfile();
        let logger = Logger::new(logger_path)?;

        Ok(Server {
            dir,
            port,
            threadpool_size,
            logger,
        })
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

    pub fn log(&mut self, msg: String) -> Result<(), Error> {
        self.logger.log(msg.as_bytes())?;
        Ok(())
    }
}
