#[derive(Debug)]
pub struct Config {
    verbose: usize,
    port: String,
    timeout: u64,
    dbfilename: String,
    logfile: String,
}

impl Config {
    pub fn new(verbose:usize, port:String, timeout:u64, dbfilename:String,
        logfile:String) -> Self {
        //hardcodeo configuracion
        let verbose = verbose;
        let port = port;
        let timeout = timeout;
        let dbfilename = dbfilename;
        let logfile = logfile;
        Config {
            verbose,
            port,
            timeout,
            dbfilename,
            logfile,
        }
    }

    pub fn get_verbose(self) -> usize {
        self.verbose
    }
    pub fn get_port(self) -> String {
        self.port
    }
    pub fn get_timeout(self) -> u64 {
        self.timeout
    }
    pub fn get_dbfilename(self) -> String {
        self.dbfilename
    }
    pub fn get_logfile(self) -> String {
        self.logfile
    }

}
