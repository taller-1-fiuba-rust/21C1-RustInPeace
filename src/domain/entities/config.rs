use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Config {
    config: HashMap<String, String>,
    path: String,
}

impl Config {
    pub fn new(path: String) -> Self {
        let config = lines_from_file(&path).unwrap();

        Config { path, config }
    }

    pub fn get_attribute(&self, attribute: String) -> Result<&String, Error> {
        if let Some(value) = self.config.get(&attribute) {
            return Ok(value);
        } else {
            return Err(Error::from(ErrorKind::NotFound));
        }
    }

    pub fn set_attribute(&mut self, attribute: String, value: String) -> Result<(), Error> {
        let mut file = File::open(&self.path)?;
        let contents = format!("{} {}", &attribute, &value);
        file.write_all(contents.as_bytes())?;
        self.config.entry(attribute).or_insert(value);
        Ok(())
    }
}

pub fn lines_from_file(path: &String) -> Result<HashMap<String, String>, Error> {
    let file = File::open(path)?;
    let f = BufReader::new(file);
    let lines: Vec<String> = f.lines().collect::<Result<_, _>>()?;
    let mut map = HashMap::new();
    for line in lines {
        let vec_aux: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();
        if vec_aux.len() == 2 {
            map.entry(vec_aux[0].clone()).or_insert(vec_aux[1].clone());
        }
    }
    Ok(map)
}
