#[derive(Debug)]
pub struct Config {
    verbose: usize,
    port: String,
    timeout: u64,
    dbfilename: String,
    logfile: String,
}

impl Config {
    pub fn new() -> Self {
        //hardcodeo configuracion
        let verbose = "1".parse().unwrap(); //to handle
        let port = "8080".to_string();
        let timeout = "10".parse().unwrap(); //to handle
        let dbfilename = "custom.rbd".to_string();
        let logfile = "log.out".to_string();
        Config {
            verbose,
            port,
            timeout,
            dbfilename,
            logfile,
        }
    }

    pub fn get_port(self) -> String {
        self.port
    }
}
