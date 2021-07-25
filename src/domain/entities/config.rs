//! Lee y actualiza los parámetros de configuración del servidor.

use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;
use std::io::{BufRead, BufReader};

/// Esta estructura representa a la configuración establecida en el archivo "redis.conf".
/// Se compone por un HashMap donde cada clave es un parámetro de configuración y su valor, el valor establecido en el archivo.
/// Mediante los comandos `config get` y `config set` el usuario puede editar la configuración default.
/// Al actualizar un parámetro, inmediatamente se actualizará el archivo de configuración, manteniendo la consistencia entre esta estructura y el archivo.
#[derive(Debug)]
pub struct Config {
    config: HashMap<String, String>,
    path: String,
}

impl Config {
    /// Crea un `Config` con los parámetros especificados en el archivo que se encuentra en `path`.
    ///
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::entities::config::Config;
    ///
    /// # std::fs::File::create("config.txt").unwrap();
    /// let mut config = Config::new("config.txt".to_string());
    /// # std::fs::remove_file("config.txt").unwrap();
    /// ```
    pub fn new(path: String) -> Self {
        let config =
            lines_from_file(&path).expect("Couldn't read lines from config file. Aborting..");
        Config { config, path }
    }

    /// Retorna el valor configurado en `attribute`.
    ///
    /// Si el atributo no existe, devuelve error `NotFound`.
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::entities::config::Config;
    ///
    /// # std::fs::File::create("config_get.txt").unwrap();
    /// let mut config = Config::new("config_get.txt".to_string());
    /// # config.set_attribute("maxmemory".to_string(), "2mb".to_string());
    ///
    /// assert_eq!(String::from("2mb"), config.get_attribute("maxmemory".to_string()).unwrap());
    /// # std::fs::remove_file("config_get.txt").unwrap();
    /// ```
    pub fn get_attribute(&self, attribute: String) -> Result<String, Error> {
        if let Some(value) = self.config.get(&attribute) {
            Ok(value.to_string())
        } else {
            Err(Error::from(ErrorKind::NotFound))
        }
    }

    /// Retorna un `HashMap` con todos los parámetros configurados.
    ///
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::entities::config::Config;
    ///
    /// # std::fs::File::create("config_get_all.txt").unwrap();
    /// let mut config = Config::new("config_get_all.txt".to_string());
    /// # config.set_attribute("maxmemory".to_string(), "2mb".to_string());
    /// # config.set_attribute("verbose".to_string(), "1".to_string());
    /// let attributes = config.get_all_attributes();
    ///
    /// assert_eq!(&String::from("2mb"), attributes.get("maxmemory").unwrap());
    /// assert_eq!(&String::from("1"), attributes.get("verbose").unwrap());
    /// # std::fs::remove_file("config_get_all.txt").unwrap();
    /// ```
    pub fn get_all_attributes(&self) -> &HashMap<String, String> {
        &self.config
    }

    /// Actualiza o crea un parámetro configurable.
    ///
    /// Si el atributo ya existe en la configuración, entonces lo actualiza con el valor especificado. Si no existe, lo agrega.
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::entities::config::Config;
    ///
    /// # std::fs::File::create("config_set.txt").unwrap();
    /// let mut config = Config::new("config_set.txt".to_string());
    /// config.set_attribute("maxmemory".to_string(), "2mb".to_string());
    /// let attributes = config.get_all_attributes();
    ///
    /// assert_eq!(&String::from("2mb"), attributes.get("maxmemory").unwrap());
    ///
    /// config.set_attribute("maxmemory".to_string(), "3mb".to_string());
    /// let attributes = config.get_all_attributes();
    ///
    /// assert_eq!(&String::from("3mb"), attributes.get("maxmemory").unwrap());
    /// # std::fs::remove_file("config_set.txt").unwrap();
    /// ```
    pub fn set_attribute(&mut self, attribute: String, value: String) -> Result<(), Error> {
        let entry = self.config.entry(attribute.clone());
        if let std::collections::hash_map::Entry::Occupied(mut e) = entry {
            e.insert(value);
        } else {
            self.config.entry(attribute).or_insert(value);
        }
        self.update_file()?;
        Ok(())
    }

    /// Actualiza el archivo ubicado en `path`.
    ///
    /// Escribe en el archivo de configuración ubicado en `path` el contenido del HashMap `config`.
    /// Devuelve Error si ocurre un error inesperado al crear el archivo o al escribir sobre él.
    /// ```
    /// use proyecto_taller_1::domain::entities::config::Config;
    ///
    /// # std::fs::File::create("config_update.txt").unwrap();
    /// let mut config = Config::new("config_update.txt".to_string());
    /// config.update_file();
    /// # std::fs::remove_file("config_update.txt").unwrap();
    /// ```
    pub fn update_file(&mut self) -> Result<(), Error> {
        let mut file = File::create(&self.path)?;
        let mut contents: String = String::from("");
        for (k, value) in &self.config {
            contents += &format!("{} {}\n", k, value);
        }
        file.write_all(contents.as_bytes())?;
        Ok(())
    }
}

/// Lee las líneas del archivo ubicado en `path` y retorna un `HashMap`.
///
/// Lee cada línea en `path` de la forma `clave valor` y las guarda en un HashMap.
/// Retorna error si el archivo no existe o si falla la obtención de las líneas del archivo.
/// # Example
/// ```
/// use proyecto_taller_1::domain::entities::config::lines_from_file;
/// use std::io::Write;
///
/// let mut file = std::fs::File::create("config_lines.txt").unwrap();
/// file.write_all(format!("key value\nverbose 1\n").as_bytes()).unwrap();
/// let hashmap = lines_from_file("config_lines.txt").unwrap();
///
/// assert_eq!(hashmap.get("key").unwrap(), &"value".to_string());
/// assert_eq!(hashmap.get("verbose").unwrap(), &"1".to_string());
/// # std::fs::remove_file("config_lines.txt").unwrap();
/// ```
pub fn lines_from_file(path: &str) -> Result<HashMap<String, String>, Error> {
    let file = File::open(path)?;
    let f = BufReader::new(file);
    let lines: Vec<String> = f.lines().collect::<Result<_, _>>()?;
    let mut map = HashMap::new();
    for line in lines {
        let vec_aux: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();
        if vec_aux.len() == 2 {
            map.entry(vec_aux[0].clone())
                .or_insert_with(|| vec_aux[1].clone());
        }
    }
    Ok(map)
}

#[test]
fn test_01_config_sets_one_new_attribute_value() {
    std::fs::File::create("./src/dummy_redis.txt").unwrap();
    let mut config = Config::new(String::from("./src/dummy_redis.txt"));
    config
        .set_attribute(String::from("maxmemory"), String::from("2mb"))
        .unwrap();
    assert!(config.config.contains_key("maxmemory"));
    assert_eq!(
        config.config.get("maxmemory").unwrap(),
        &String::from("2mb")
    );
    std::fs::remove_file("./src/dummy_redis.txt").unwrap();
}

#[test]
fn test_02_config_sets_multiple_new_attribute_value() {
    std::fs::File::create("./src/dummy_redis2.txt").unwrap();
    let mut config = Config::new(String::from("./src/dummy_redis2.txt"));
    config
        .set_attribute(String::from("maxmemory"), String::from("2mb"))
        .unwrap();
    assert!(config.config.contains_key("maxmemory"));
    assert_eq!(
        config.config.get("maxmemory").unwrap(),
        &String::from("2mb")
    );
    config
        .set_attribute(
            String::from("maxmemory-policy"),
            String::from("allkeys-lru"),
        )
        .unwrap();
    config
        .set_attribute(String::from("maxmemory"), String::from("3mb"))
        .unwrap();
    assert!(config.config.contains_key("maxmemory-policy"));
    assert_eq!(
        config.config.get("maxmemory-policy").unwrap(),
        &String::from("allkeys-lru")
    );
    assert!(config.config.contains_key("maxmemory"));
    assert_eq!(
        config.config.get("maxmemory").unwrap(),
        &String::from("3mb")
    );
    std::fs::remove_file("./src/dummy_redis2.txt").unwrap();
}
