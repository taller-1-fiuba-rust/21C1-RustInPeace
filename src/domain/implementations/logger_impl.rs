//! Imprime información sobre el servidor en un archivo exclusivo.

use std::fs::File;
use std::io::Error;
use std::io::Write;

/// La estructura representa un archivo donde se reúne información sobre la ejecución del servidor.
#[derive(Debug)]
pub struct Logger {
    file: File,
}

impl Logger {
    /// Crea una instancia de Logger a partir de la dirección del archivo.
    ///
    /// ```
    /// use proyecto_taller_1::domain::implementations::logger_impl::Logger;
    ///
    /// let logger = Logger::new("dummy_logger");
    /// # std::fs::remove_file("dummy_logger");
    /// ```
    pub fn new(path: &str) -> Result<Self, Error> {
        let file = File::create(path)?;
        Ok(Logger { file })
    }

    /// Escribe un mensaje sobre el archivo de log.
    ///
    /// ```
    /// use proyecto_taller_1::domain::implementations::logger_impl::Logger;
    ///
    /// let mut logger = Logger::new("dummy_logger").unwrap();
    /// logger.log("log some message".as_bytes());
    /// # std::fs::remove_file("dummy_logger");
    /// ```
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
