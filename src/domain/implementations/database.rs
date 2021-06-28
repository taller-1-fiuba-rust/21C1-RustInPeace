use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};
use regex::Regex;
use std::collections::HashMap;

#[allow(unused)]
use crate::domain::entities::key_value_item::KeyAccessTime;

#[allow(unused)]
use crate::domain::entities::key_value_item_serialized::KeyValueItemSerialized;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::Write;
use std::io::{self};
use std::num::ParseIntError;
use std::path::Path;

#[derive(Debug)]
pub struct Database {
    dbfilename: String,
    items: HashMap<String, ValueTimeItem>,
}

impl Database {
    pub fn new(filename: String) -> Database {
        let mut db = Database {
            dbfilename: filename,
            items: HashMap::new(),
        };
        db.load_items();
        db
    }
    pub fn _get_filename(&self) -> String {
        self.dbfilename.clone()
    }
    /// devuelve el hashmap donde se almacenan los datos
    pub fn _get_items(&self) -> &HashMap<String, ValueTimeItem> {
        &self.items
    }
    /// borra todos las claves (y sus valores asociados) de la base de datos
    pub fn clean_items(&mut self) -> &HashMap<String, ValueTimeItem> {
        self.items.clear();
        &self.items
    }
    /// devuelve el valor almacenado en **key**
    pub fn search_item_by_key(&self, key: String) -> Option<&ValueTimeItem> {
        self.items.get(&key)
    }
    /// Devuelve las claves que hacen *match* con un *pattern* sin uso de regex (limitado)
    pub fn get_keys_that_match_pattern_sin_regex(&self, pattern: String) -> Vec<String> {
        let mut vector_keys = vec![];
        for key in &self.items {
            let current_key = key.to_owned().0.to_string();
            vector_keys.push(current_key);
        }
        let mut vector_keys_filtered = vec![];
        for key in vector_keys {
            if key.contains(&pattern.to_string()) {
                vector_keys_filtered.push(key);
            }
        }
        vector_keys_filtered
    }
    /// Devuelve las claves que hacen *match* con un *pattern* con uso de regex (exhaustivo)
    pub fn get_keys_that_match_pattern(&self, pattern: &str) -> Vec<String> {
        let mut vec_matching_keys = vec![];
        //aca agarro todas las claves disponibles en un vector
        let mut vector_keys = vec![];
        for key in &self.items {
            let current_key = key.to_owned().0.to_string();
            vector_keys.push(current_key);
        }
        //aca genero el regex a partir de pattern y lo comparo contra todas las claves
        let re = Regex::new(pattern).unwrap();
        for key in vector_keys {
            if re.is_match(&key) {
                vec_matching_keys.push(key);
            }
        }
        vec_matching_keys
    }

    ///devuelve **true** si la clave existe en *database*
    pub fn key_exists(&self, key: String) -> bool {
        self.items.contains_key(&key)
    }
    /// permite agregar *clave* y *valor* a la base de datos
    pub fn add(&mut self, key: String, value: ValueTimeItem) {
        self.items.insert(key, value);
    }
    /// obtiene las claves de la **db** que hacen *match* con el **pat** + **element** (de
    /// **elements** y devuelve una tupla con (**element**,**patterned_key_value**)
    pub fn get_values_of_external_keys_that_match_a_pattern(
        &self,
        elements: Vec<String>,
        pat: &str,
    ) -> Option<Vec<(String, String)>> {
        let mut vec_auxiliar = Vec::new();
        for element in elements {
            let patterned_key = pat.to_string() + element.as_str();
            if self.items.contains_key(&patterned_key) {
                let current_value = self
                    .items
                    .get(&patterned_key)
                    .unwrap()
                    .get_value_version_2()
                    .unwrap();
                let vectorcito = (element, current_value[0].to_string());
                vec_auxiliar.push(vectorcito);
            }
        }
        if !vec_auxiliar.is_empty() {
            Some(vec_auxiliar)
        } else {
            None
        }
    }
    // VER EL TEMA DE LOS TIPOS DE DATOS GUARDADOS EN VALUE (SIN ITEM) PORQUE PUEDE SER CUALQUIERA DE 3 TIPOS
    /// Resetea el tiempo de acceso **KeyAccessTime** de una clave
    pub fn reboot_time(&mut self, key: String) {
        let current_value = self.items.remove(&key).unwrap();
        let cv = current_value.get_value_version_2().unwrap();
        let mut vec_aux = vec![];
        for elemento in cv {
            vec_aux.push(elemento.to_string());
        }
        let vt = ValueTimeItem {
            value: ValueType::ListType(vec_aux),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        self.items.insert(key, vt);
    }

    pub fn get_type_of_value(&self, key: String) -> String {
        self.items.get(&key).unwrap().get_value_type()
    }

    pub fn copy(&mut self, source: String, destination: String, replace: bool) -> Option<()> {
        let source_item = self.items.get(&source);
        let new_value = source_item?.get_copy_of_value();

        if self.items.contains_key(&destination) {
            if replace {
                let dest = self.items.get_mut(&destination).unwrap();
                dest._set_value(new_value);
                return Some(());
            } else {
                return None;
            }
        }
        // } else {
        //ver set del tiempo cuando es nuevo
        self.items
            .entry(destination)
            .or_insert_with(|| ValueTimeItem::new(new_value, KeyAccessTime::Volatile(12423423)));
        Some(())
        // }
    }

    pub fn persist(&mut self, key: String) -> bool {
        match self.items.get_mut(&key) {
            Some(item) => item.make_persistent(),
            None => false,
        }
    }

    ///renombra una clave, conservando su valor actual
    pub fn rename_key(&mut self, current_key: String, new_key: String) -> bool {
        let item = self.items.remove(&current_key);
        if let Some(item) = item {
            self.items.insert(new_key, item);
            true
        } else {
            false
        }
    }
    //------------------------------------------------
    pub fn append_string(&mut self, key: &str, string: &str) -> usize {
        match self.items.get_mut(&key.to_string()) {
            Some(item) => {
                if let ValueType::StringType(old_value) = item.get_copy_of_value() {
                    let len = old_value.len() + string.len();
                    let new_value = ValueType::StringType(old_value + string);
                    item._set_value(new_value);
                    len
                } else {
                    0
                }
            }
            None => {
                self.items.insert(
                    key.to_string(),
                    ValueTimeItem::new(
                        ValueType::StringType(string.to_string()),
                        KeyAccessTime::Volatile(3423423),
                    ),
                );
                string.len()
            }
        }
    }

    pub fn decrement_key_by(&mut self, key: &str, decr: i64) -> Result<i64, ParseIntError> {
        match self.items.get_mut(&key.to_string()) {
            Some(item) => {
                if let ValueType::StringType(str) = item.get_copy_of_value() {
                    let str_as_number = str.parse::<i64>()?;
                    let new_value = ValueType::StringType((str_as_number - decr).to_string());
                    item._set_value(new_value);
                    Ok(str_as_number - decr)
                } else {
                    //hay que devolver algo posta aca
                    Ok(1)
                }
            }
            None => {
                let new_value = 0 - decr;
                self.items.insert(
                    key.to_string(),
                    ValueTimeItem::new(
                        ValueType::StringType(new_value.to_string()),
                        KeyAccessTime::Volatile(3423423),
                    ),
                );
                Ok(new_value)
            }
        }
    }

    pub fn increment_key_by(&mut self, key: &str, incr: i64) -> Result<i64, ParseIntError> {
        let item = self.items.get_mut(&key.to_string()).unwrap();
        if let ValueType::StringType(str) = item.get_copy_of_value() {
            let str_as_number = str.parse::<i64>()?;
            let new_value = ValueType::StringType((str_as_number + incr).to_string());
            item._set_value(new_value);
            return Ok(str_as_number + incr);
        } else {
            //devolver error
        }
        let new_value = incr;
        self.items.insert(
            key.to_string(),
            ValueTimeItem::new(
                ValueType::StringType(new_value.to_string()),
                KeyAccessTime::Volatile(3423423),
            ),
        );
        Ok(new_value)
    }

    /// REVISAR: NO ES LO MISMO QUE search_item_by_key?
    pub fn get_value_by_key(&self, key: &str) -> Option<String> {
        let item = self.items.get(&key.to_string());
        if let Some(item) = item {
            let value = item.get_copy_of_value();
            if let ValueType::StringType(str) = value {
                Some(str)
            } else {
                None
            }
        } else {
            None
        }
    }

    //agregar tests
    pub fn get_strlen_by_key(&self, key: &str) -> Option<usize> {
        let item = self.items.get(&key.to_string());
        if let Some(item) = item {
            let value = item.get_copy_of_value();
            if let ValueType::StringType(str) = value {
                Some(str.len())
            } else {
                None
            }
        } else {
            Some(0)
        }
    }

    //agregar tests
    pub fn getdel_value_by_key(&mut self, key: &str) -> Option<String> {
        let item = self.items.get(&key.to_string());
        if let Some(item) = item {
            let value = item.get_copy_of_value();
            if let ValueType::StringType(str) = value {
                self.delete_key(key.to_string());
                Some(str)
            } else {
                None
            }
        } else {
            None
        }
    }

    //agregar tests
    pub fn getset_value_by_key(&mut self, key: &str, new_value: &str) -> Option<String> {
        let item = self.items.get_mut(&key.to_string());
        if let Some(item) = item {
            let value = item.get_copy_of_value();
            if let ValueType::StringType(str) = value {
                item._set_value(ValueType::StringType(new_value.to_string()));
                // self.replace_value_on_key(
                //     key.to_string(),
                //     ValueType::StringType(new_value.to_string()),
                // );
                Some(str)
            } else {
                //error
                None
            }
        } else {
            //nil
            None
        }
    }

    /* Si el servidor se reinicia se deben cargar los items del file */
    pub fn load_items(&mut self) {
        if let Ok(lines) = Database::read_lines(self.dbfilename.to_string()) {
            for line in lines {
                if let Ok(kvi_serialized) = line {
                    let kvis = KeyValueItemSerialized::new(kvi_serialized);
                    let item = kvis.transform_to_item();
                    self.items.insert(item.0, item.1);
                } else {
                    panic!("Error al leer línea del archivo:");
                }
            }
        } else {
            panic!("Error al leer el archivo dump");
        }
    }

    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let path = Path::new(filename.as_ref());
        let file = File::open(&path);
        match file {
            Ok(_) => Ok(io::BufReader::new(file.unwrap()).lines()),
            Err(_) => {
                let mut _file = File::create(&path)?; //Lo crea en write-only mode.
                let path = Path::new(filename.as_ref());
                let file_op = File::open(&path);
                Ok(io::BufReader::new(file_op.unwrap()).lines())
            }
        }
    }
    /*
      Guarda cada item que tiene en memoria, en el formato adecuado para la serialización.
      Formato: key;last_access_time;type;value1,value,2
    */
    pub fn save_items_to_file(&self) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create_new(false)
            .open(self.dbfilename.to_string())
            .unwrap();

        for kvi in &self.items {
            let kvi_type = match kvi.1.get_value() {
                ValueType::StringType(_) => "string",
                ValueType::SetType(_) => "set",
                ValueType::ListType(_) => "list",
            };
            writeln!(
                file,
                "{};{};{};{}",
                kvi.0,
                kvi.1.get_last_access_time().to_string(),
                kvi_type,
                kvi.1.get_value().to_string()
            )
            .unwrap();
        }
    }
    /// devuelve todos los valores almacenados en todas las claves en orden aleatorio
    pub fn _get_all_values(&self) -> Vec<ValueType> {
        let mut all_values = Vec::new();
        let values_vector = &self.items;
        for value in values_vector.values() {
            let only_value = value.get_copy_of_value();
            all_values.push(only_value);
        }
        all_values
    }

    /// devuelve la cantidad de claves almacenadas en la base de datos
    pub fn get_size(&self) -> usize {
        self.items.len()
    }

    /// permite eliminar una clave y su valor asociado
    pub fn delete_key(&mut self, key: String) -> bool {
        matches!(self.items.remove(&key), Some(_key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::key_value_item::{KeyAccessTime, ValueType};
    use std::io::BufReader;

    #[test]
    fn test_00_filter_keys_by_pattern() {
        let mut db = Database::new(String::from("./src/dummy.txt"));
        let vt_1 = ValueTimeItem {
            value: ValueType::StringType("valor_1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_2 = ValueTimeItem {
            value: ValueType::StringType("valor_2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_3 = ValueTimeItem {
            value: ValueType::StringType("valor_3".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_4 = ValueTimeItem {
            value: ValueType::StringType("valor_4".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        db.items.insert("weight_bananas".to_string(), vt_1);
        db.items.insert("apples_weight".to_string(), vt_2);
        db.items
            .insert("deliciosos_kiwi_weight_baratos".to_string(), vt_3);
        db.items.insert("banana_weight".to_string(), vt_4);

        // db.get_values_of_external_keys_that_match_a_pattern("banana");
        let vec_filtered = db.get_keys_that_match_pattern_sin_regex("weight".to_string());
        assert_eq!(vec_filtered.len(), 4);
        let _ = std::fs::remove_file("./src/dummy.txt");
    }

    #[test]
    fn empty_database_returns_cero() {
        let db = Database {
            dbfilename: "file".to_string(),
            items: HashMap::new(),
        };

        assert_eq!(db.get_size(), 0);
    }

    #[test]
    fn test_01_database_copies_value_to_new_key() {
        let mut db = Database::new(String::from("./src/dummy.txt"));
        db.add(
            "clave_1".to_string(),
            ValueTimeItem {
                value: (ValueType::StringType("valor_1".to_string())),
                last_access_time: KeyAccessTime::Persistent,
            },
        );

        let source = String::from("clave_1");
        let destination = String::from("clone");
        assert_eq!(db.copy(source, destination, false).unwrap(), ());

        let new_item = db.search_item_by_key(String::from("clone")).unwrap();
        if let ValueType::StringType(str) = new_item.get_value() {
            assert_eq!(str, &String::from("valor_1"));
        }
        let _ = std::fs::remove_file("./src/dummy.txt");
    }

    #[test]
    fn test_02_database_copy_replaces_key_with_new_value() {
        let mut db = Database::new(String::from("./src/dummy2.txt"));
        db.add(
            "clave_1".to_string(),
            ValueTimeItem {
                value: (ValueType::StringType("valor_1".to_string())),
                last_access_time: KeyAccessTime::Persistent,
            },
        );
        db.add(
            "clave_2".to_string(),
            ValueTimeItem {
                value: (ValueType::StringType("valor_2".to_string())),
                last_access_time: KeyAccessTime::Persistent,
            },
        );

        let source = String::from("clave_1");
        let destination = String::from("clone");
        assert_eq!(db.copy(source, destination, false).unwrap(), ());

        let new_item = db.search_item_by_key(String::from("clone")).unwrap();
        if let ValueType::StringType(str) = new_item.get_value() {
            assert_eq!(str, &String::from("valor_1"));
        }

        let source = String::from("clave_2");
        let destination = String::from("clone");
        assert_eq!(db.copy(source, destination, true).unwrap(), ());

        let new_item = db.search_item_by_key(String::from("clone")).unwrap();
        if let ValueType::StringType(str) = new_item.get_value() {
            assert_eq!(str, &String::from("valor_2"));
        }
        let _ = std::fs::remove_file("./src/dummy2.txt");
    }

    #[test]
    fn test_03_clean_items_deletes_all_items() {
        let mut db = Database::new(String::from("./src/database1.txt"));
        db.clean_items();
        assert_eq!(db.get_size(), 0);
        std::fs::remove_file("./src/database1.txt").unwrap();
    }

    #[test]
    fn test_04_deletes_an_item_succesfully() {
        let mut db = Database::new("file2".to_string());

        let vt_1 = ValueTimeItem {
            value: ValueType::StringType("valor_1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_2 = ValueTimeItem {
            value: ValueType::StringType("valor_2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        db.items.insert("weight_bananas".to_string(), vt_1);
        db.items.insert("apples_weight".to_string(), vt_2);

        db.delete_key("apples_weight".to_string());

        assert_eq!(db.get_size(), 1);
        std::fs::remove_file("file2").unwrap();
    }

    #[test]
    fn test_05_persist_changes_type_of_access_time() {
        use crate::domain::entities::key_value_item::KeyAccessTime;
        let mut db = Database::new("file".to_string());

        let vt_1 = ValueTimeItem {
            value: ValueType::StringType("valor_1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_2 = ValueTimeItem {
            value: ValueType::StringType("valor_2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        db.items.insert("weight_bananas".to_string(), vt_1);
        db.items.insert("apples_weight".to_string(), vt_2);
        //--------
        let _res = db.persist("weight_bananas".to_string());

        let item = db.items.get(&"weight_bananas".to_string()).unwrap();
        match *item.get_last_access_time() {
            KeyAccessTime::Persistent => assert!(true),
            KeyAccessTime::Volatile(_tmt) => assert!(false),
        }

        std::fs::remove_file("file").unwrap();
    }

    #[test]
    fn add_item() {
        let mut db = Database {
            dbfilename: "file".to_string(),
            items: HashMap::new(),
        };
        db.add(
            String::from("nueva_key"),
            ValueTimeItem {
                value: (ValueType::StringType(String::from("222"))),
                last_access_time: KeyAccessTime::Persistent,
            },
        );

        assert_eq!(
            db.items.get("nueva_key").unwrap().value.to_string(),
            String::from("222")
        );
        assert_eq!(db.items.len(), 1)
    }

    #[test]
    fn delete_item() {
        let mut db = Database {
            dbfilename: "file".to_string(),
            items: HashMap::new(),
        };
        db.items.insert(
            String::from("nueva_key"),
            ValueTimeItem {
                value: ValueType::StringType(String::from("222")),
                last_access_time: KeyAccessTime::Persistent,
            },
        );

        assert_eq!(db.items.len(), 1);
        db.delete_key(String::from("nueva_key"));
        assert_eq!(db.items.len(), 0);
    }

    #[test]
    fn filename_is_correct() {
        let db = Database {
            dbfilename: "file".to_string(),
            items: HashMap::new(),
        };
        assert_eq!(db._get_filename(), "file".to_string());
    }

    #[test]
    fn load_items_from_file() {
        let mut file = File::create("file_5".to_string()).expect("Unable to open");
        file.write_all(b"124key;1623433677;string;value2\n")
            .unwrap();

        let db = Database::new("file_5".to_string());
        assert_eq!(db.items.len(), 1);
        let mut iter = db.items.iter();
        let kvi = iter.next().unwrap();

        assert_eq!(kvi.0, "124key");
        assert_eq!(kvi.1.value.to_string(), String::from("value2"));
        match kvi.1.last_access_time {
            KeyAccessTime::Volatile(1623433677) => assert!(true),
            _ => assert!(false),
        }
        let _ = std::fs::remove_file("file_5");
    }

    #[test]
    fn create_database_file() {
        assert!(!std::path::Path::new("new_file").exists());
        let _db = Database::new("new_file".to_string());
        assert!(std::path::Path::new("new_file").exists());
        let _ = std::fs::remove_file("new_file");
    }

    #[test]
    fn save_items_to_file() {
        let mut db = Database::new("file".to_string());

        let list = vec![
            "un_item_string".to_string(),
            "segundo_item_list_string".to_string(),
        ];

        db.items.insert(
            "clave_2".to_string(),
            ValueTimeItem {
                value: ValueType::ListType(list),
                last_access_time: KeyAccessTime::Volatile(1231230),
            },
        );

        db.save_items_to_file();

        let file = File::open(&db.dbfilename);
        let reader = BufReader::new(file.unwrap());
        let mut it = reader.lines();

        match it.next().unwrap() {
            Ok(t) => assert_eq!(
                t,
                "clave_2;1231230;list;un_item_string,segundo_item_list_string"
            ),
            _ => assert!(false),
        }

        let _ = std::fs::remove_file("file");
    }

    #[test]
    fn test_08_size_in_memory_is_correct() {
        let mut db = Database::new("file1".to_string());

        let vt_1 = ValueTimeItem {
            value: ValueType::StringType("valor_1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_2 = ValueTimeItem {
            value: ValueType::StringType("valor_2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        db.items.insert("weight_bananas".to_string(), vt_1);
        db.items.insert("apples_weight".to_string(), vt_2);
        std::fs::remove_file("file1").unwrap();
    }

    #[test]
    fn test_09_persist_changes_type_of_access_time() {
        use crate::domain::entities::key_value_item::KeyAccessTime;
        let mut db = Database::new(String::from("./src/dummy_persist.txt"));
        let _res = db.items.insert(
            "clave_1".to_string(),
            ValueTimeItem::new(
                ValueType::StringType("value".to_string()),
                KeyAccessTime::Persistent,
            ),
        );

        let item = db.items.get("clave_1").unwrap();
        match *item.get_last_access_time() {
            KeyAccessTime::Persistent => assert!(true),
            KeyAccessTime::Volatile(_tmt) => assert!(false),
        }
        std::fs::remove_file("./src/dummy_persist.txt".to_string()).unwrap();
    }

    #[test]
    fn test_10_append_adds_string_to_end_of_existing_value() {
        let mut db = Database::new(String::from("./src/dummy_appends_2.txt"));
        let _res = db.items.insert(
            "mykey".to_string(),
            ValueTimeItem::new(
                ValueType::StringType("Hello".to_string()),
                KeyAccessTime::Persistent,
            ),
        );

        let len = db.append_string(&"mykey".to_string(), &" World".to_string());
        assert_eq!(len, 11);
        std::fs::remove_file("./src/dummy_appends_2.txt".to_string()).unwrap();
    }

    #[test]
    fn test_11_append_adds_string_to_new_value() {
        let mut db = Database::new(String::from("./src/dummy_appends_1.txt"));

        let len = db.append_string(&"mykey".to_string(), &" World".to_string());
        assert_eq!(len, 6);
        std::fs::remove_file("./src/dummy_appends_1.txt".to_string()).unwrap();
    }

    #[test]
    fn test_12_decr_key_to_existing_key() {
        let mut db = Database::new(String::from("./src/dummy_decr_1.txt"));
        let _res = db.items.insert(
            "mykey".to_string(),
            ValueTimeItem::new(
                ValueType::StringType("10".to_string()),
                KeyAccessTime::Persistent,
            ),
        );

        let res = db.decrement_key_by(&"mykey".to_string(), 3).unwrap();
        assert_eq!(res, 7);
        std::fs::remove_file("./src/dummy_decr_1.txt".to_string()).unwrap();
    }

    #[test]
    fn test_13_decr_by_to_new_key() {
        let mut db = Database::new(String::from("./src/dummy_decr.txt"));

        let res = db.decrement_key_by(&"mykey".to_string(), 3).unwrap();
        assert_eq!(res, -3);
        std::fs::remove_file("./src/dummy_decr.txt".to_string()).unwrap();
    }

    #[test]
    fn test_14_decr_by_to_invalid_string_value() {
        let mut db = Database::new(String::from("./src/dummy_decr_2.txt"));
        let _res = db.items.insert(
            "mykey".to_string(),
            ValueTimeItem::new(
                ValueType::StringType("Hello".to_string()),
                KeyAccessTime::Persistent,
            ),
        );

        let res = db.decrement_key_by(&"mykey".to_string(), 3);
        assert!(res.is_err());
        let _ = std::fs::remove_file("./src/dummy_decr_2.txt".to_string());
    }

    #[test]
    fn test_15_se_obtienen_valores_de_claves_externas_a_partir_de_un_patron_y_una_lista_de_elementos(
    ) {
        let mut db = Database::new("file10".to_string());

        let vt_1 = ValueTimeItem {
            value: ValueType::StringType("1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_2 = ValueTimeItem {
            value: ValueType::StringType("2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_3 = ValueTimeItem {
            value: ValueType::StringType("11".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_4 = ValueTimeItem {
            value: ValueType::StringType("5".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        db.items.insert("weight_bananas".to_string(), vt_1);
        db.items.insert("weight_apples".to_string(), vt_2);
        db.items.insert("weight_kiwi".to_string(), vt_3);
        db.items.insert("weight_pear".to_string(), vt_4);

        let pat = "weight_";
        let vec_strings = vec![
            "sandia".to_string(),
            "pear".to_string(),
            "apples".to_string(),
        ];
        let tuplas = db.get_values_of_external_keys_that_match_a_pattern(vec_strings, pat);
        let algo = tuplas.unwrap();
        println!("{:?}", algo);
        let _removed = std::fs::remove_file("file10".to_string());
    }

    #[test]
    fn test_16_se_obtienen_keys_que_contienen_patron_regex_con_signo_de_pregunta() {
        let mut db = Database::new("file10".to_string());

        let vt_1 = ValueTimeItem {
            value: ValueType::StringType("1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_2 = ValueTimeItem {
            value: ValueType::StringType("2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_3 = ValueTimeItem {
            value: ValueType::StringType("11".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_4 = ValueTimeItem {
            value: ValueType::StringType("5".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_5 = ValueTimeItem {
            value: ValueType::StringType("1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_6 = ValueTimeItem {
            value: ValueType::StringType("2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_7 = ValueTimeItem {
            value: ValueType::StringType("11".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_8 = ValueTimeItem {
            value: ValueType::StringType("5".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        db.items.insert("pablo".to_string(), vt_1);
        db.items.insert("juan".to_string(), vt_2);
        db.items.insert("mariana".to_string(), vt_3);
        db.items.insert("lucia".to_string(), vt_4);
        db.items.insert("mariano".to_string(), vt_5);
        db.items.insert("meriana".to_string(), vt_6);
        db.items.insert("miriana".to_string(), vt_7);
        db.items.insert("luciana".to_string(), vt_8);

        let pat = "m?riana";
        let matching_keys = db.get_keys_that_match_pattern(pat);
        //let algo = tuplas.unwrap();
        for key in matching_keys {
            println!("{:?}", key)
        }
        let _removed = std::fs::remove_file("file10".to_string());
    }

    #[test]
    fn test_17_se_obtienen_keys_que_contienen_patron_regex_solo_exp_entre_corchetes() {
        let mut db = Database::new("file10".to_string());

        let vt_1 = ValueTimeItem {
            value: ValueType::StringType("1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_2 = ValueTimeItem {
            value: ValueType::StringType("2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_3 = ValueTimeItem {
            value: ValueType::StringType("11".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_4 = ValueTimeItem {
            value: ValueType::StringType("5".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_5 = ValueTimeItem {
            value: ValueType::StringType("1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_6 = ValueTimeItem {
            value: ValueType::StringType("2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_7 = ValueTimeItem {
            value: ValueType::StringType("11".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_8 = ValueTimeItem {
            value: ValueType::StringType("5".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        db.items.insert("mia".to_string(), vt_1);
        db.items.insert("juan".to_string(), vt_2);
        db.items.insert("mariana".to_string(), vt_3);
        db.items.insert("lucia".to_string(), vt_4);
        db.items.insert("malala".to_string(), vt_5);
        db.items.insert("meriana".to_string(), vt_6);
        db.items.insert("miriana".to_string(), vt_7);
        db.items.insert("luciana".to_string(), vt_8);

        let pat = "m[ae]riana";
        let matching_keys = db.get_keys_that_match_pattern(pat);
        //let algo = tuplas.unwrap();
        for key in matching_keys {
            println!("{:?}", key)
        }
    }

    #[test]
    fn test_18_se_obtienen_keys_que_contienen_patron_regex_excepto_exp_entre_corchetes_tipo_1() {
        let mut db = Database::new("file10".to_string());

        let vt_1 = ValueTimeItem {
            value: ValueType::StringType("1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_2 = ValueTimeItem {
            value: ValueType::StringType("2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_3 = ValueTimeItem {
            value: ValueType::StringType("11".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_4 = ValueTimeItem {
            value: ValueType::StringType("5".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_5 = ValueTimeItem {
            value: ValueType::StringType("1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_6 = ValueTimeItem {
            value: ValueType::StringType("2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_7 = ValueTimeItem {
            value: ValueType::StringType("11".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_8 = ValueTimeItem {
            value: ValueType::StringType("5".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        db.items.insert("mia".to_string(), vt_1);
        db.items.insert("juan".to_string(), vt_2);
        db.items.insert("mariana".to_string(), vt_3);
        db.items.insert("lucia".to_string(), vt_4);
        db.items.insert("malala".to_string(), vt_5);
        db.items.insert("meriana".to_string(), vt_6);
        db.items.insert("miriana".to_string(), vt_7);
        db.items.insert("luciana".to_string(), vt_8);

        let pat = "m[^a]riana";
        let matching_keys = db.get_keys_that_match_pattern(pat);
        //let algo = tuplas.unwrap();
        for key in matching_keys {
            println!("{:?}", key)
        }
        let _removed = std::fs::remove_file("file10".to_string());
    }

    #[test]
    fn test_19_se_obtienen_keys_que_contienen_patron_regex_excepto_exp_entre_corchetes_tipo_2_rango(
    ) {
        let mut db = Database::new("file10".to_string());

        let vt_1 = ValueTimeItem {
            value: ValueType::StringType("1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_2 = ValueTimeItem {
            value: ValueType::StringType("2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_3 = ValueTimeItem {
            value: ValueType::StringType("11".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_4 = ValueTimeItem {
            value: ValueType::StringType("5".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_5 = ValueTimeItem {
            value: ValueType::StringType("1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_6 = ValueTimeItem {
            value: ValueType::StringType("2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_7 = ValueTimeItem {
            value: ValueType::StringType("11".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_8 = ValueTimeItem {
            value: ValueType::StringType("5".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        db.items.insert("mia".to_string(), vt_1);
        db.items.insert("juan".to_string(), vt_2);
        db.items.insert("mariana".to_string(), vt_3);
        db.items.insert("muriana".to_string(), vt_4);
        db.items.insert("malala".to_string(), vt_5);
        db.items.insert("meriana".to_string(), vt_6);
        db.items.insert("miriana".to_string(), vt_7);
        db.items.insert("moriana".to_string(), vt_8);

        let pat = "m[a-o]riana";
        let matching_keys = db.get_keys_that_match_pattern(pat);
        //let algo = tuplas.unwrap();
        for key in matching_keys {
            println!("{:?}", key)
        }
        let _ = std::fs::remove_file("file10".to_string());
    }

    #[test]
    fn test_20_se_obtienen_keys_que_contienen_patron_regex_asterisco() {
        let mut db = Database::new("file10".to_string());

        let vt_1 = ValueTimeItem {
            value: ValueType::StringType("1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_2 = ValueTimeItem {
            value: ValueType::StringType("2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_3 = ValueTimeItem {
            value: ValueType::StringType("11".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_4 = ValueTimeItem {
            value: ValueType::StringType("5".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_5 = ValueTimeItem {
            value: ValueType::StringType("1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_6 = ValueTimeItem {
            value: ValueType::StringType("2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_7 = ValueTimeItem {
            value: ValueType::StringType("11".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_8 = ValueTimeItem {
            value: ValueType::StringType("5".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        db.items.insert("mia".to_string(), vt_1);
        db.items.insert("jose".to_string(), vt_2);
        db.items.insert("mariana".to_string(), vt_3);
        db.items.insert("pedro".to_string(), vt_4);
        db.items.insert("malala".to_string(), vt_5);
        db.items.insert("meriana".to_string(), vt_6);
        db.items.insert("miriana".to_string(), vt_7);
        db.items.insert("moriana".to_string(), vt_8);

        let pat = "m*a";
        let matching_keys = db.get_keys_that_match_pattern(pat);
        //let algo = tuplas.unwrap();
        for key in matching_keys {
            println!("{:?}", key)
        }
        let _ = std::fs::remove_file("file10".to_string());
    }
}
