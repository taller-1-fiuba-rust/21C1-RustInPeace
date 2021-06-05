use std::fs::File;
use std::io::Error;
use std::io::Write;

#[derive(Debug)]
pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(path: &str) -> Result<Self, Error> {
        let file = File::create(path)?;
        Ok(Logger { file })
    }

    pub fn log(&mut self, message: &[u8]) -> std::io::Result<()> {
        self.file.write_all(message)?;
        Ok(())
    }
}

#[test]
fn test_01_logger_logs_to_right_file() {
    use std::io::BufRead;
    use std::io::BufReader;

    let path = "./src/redis-server.log".to_string();
    let mut logger = Logger::new(&path).unwrap();
    logger.log("Logging test".as_bytes()).unwrap();
    let file = File::open(&path).unwrap();
    let f = BufReader::new(file);
    for line in f.lines() {
        assert_eq!(line.unwrap().as_bytes(), "Logging test".as_bytes());
    }
    std::fs::remove_file(path).unwrap();
}
