// use std::error::Error;
// use std::u64;
use regex::Regex;
use std::collections::HashMap; //, ops::Bound};

//use regex::{Captures, Regex};

//use crate::domain::entities::key_value_item::ValueType;
use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};

#[allow(unused)]
use crate::domain::entities::key_value_item::KeyAccessTime;

#[allow(unused)]
//use crate::domain::entities::key_value_item::{KeyValueItem, ValueType};
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
    // items: Vec<KeyValueItem>,
}

impl Database {
    pub fn new(filename: String) -> Database {
        //este hashmap lo creo de prueba
        // let aux_hashmap = HashMap::new();
        let mut db = Database {
            dbfilename: filename,
            items: HashMap::new(),
        };
        db.load_items();
        db
        // Database {
        //     dbfilename: filename,
        //     items: aux_hashmap,
        // }
    }
    pub fn _get_filename(&self) -> String {
        self.dbfilename.clone()
    }
    /// devuelve el hashmap donde se almacenan los datos
    pub fn _get_items(&self) -> &HashMap<String, ValueTimeItem> {
        // pub fn _get_items(&self) -> &Vec<KeyValueItem> {
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
        // match self.items.get(&key) {
        //     Some(item) => return item,
        //     None => None,
    }
    /// Devuelve las claves que hacen *match* con un *pattern* sin uso de regex (limitado)
    pub fn get_keys_that_match_pattern_sin_regex(&self, pattern: String) -> Vec<String> {
        //-> Some(){
        let mut vector_keys = vec![];
        for key in &self.items {
            let current_key = key.to_owned().0.to_string();
            // keys_as_string = keys_as_string + &current_key;
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
        //-> Some(){
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

        // let mut vector_keys_filtered = vec![];
        // for key in vector_keys {
        //     if key.contains(&pattern.to_string()) {
        //         vector_keys_filtered.push(key);
        //     }
        // }
        // vector_keys_filtered
    }

    ///devuelve **true** si la clave existe en *database*
    pub fn key_exists(&self, key: String) -> bool {
        self.items.contains_key(&key)
    }
    /// permite agregar *clave* y *valor* a la base de datos
    pub fn add(&mut self, key: String, value: ValueTimeItem) {
        self.items.insert(key, value);
    }

    /// permite agregar *clave* y *valor* a la base de datos
    // pub fn add_or_replace(&mut self, key: String, value: ValueTimeItem) {
    //     if self.key_exists(key) {
    //         let old_key_
    //     }
    //     self.items.insert(key, value);
    // }

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
    // let item = self.items.get_mut(&key.to_string()).unwrap();

    // if let ValueType::StringType(old_value) = item.get_copy_of_value() {
    //     let len = old_value.len() + string.len();
    //     let new_value = ValueType::StringType(old_value + string);
    //     item._set_value(new_value);
    //     return len;
    // }

    /////-------------------------------------
    // for item in self.items.iter_mut() {
    //     let k = item.get_key();
    //     if k == key {
    //         if let ValueType::StringType(old_value) = item.get_copy_of_value() {
    //             let len = old_value.len() + string.len();
    //             let new_value = ValueType::StringType(old_value + string);
    //             item.set_value(new_value);
    //             return len;
    //         }
    //     }
    // }
    // self.items.push(KeyValueItem::new(
    //     key.to_string(),
    //     ValueType::StringType(string.to_string()),
    // ));
    // string.len()

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
        // let item = self.items.get_mut(&key.to_string()).unwrap();
        // if let ValueType::StringType(str) = item.get_copy_of_value() {
        //     let str_as_number = str.parse::<i64>()?;
        //     let new_value = ValueType::StringType((str_as_number - decr).to_string());
        //     item._set_value(new_value);
        //     return Ok(str_as_number - decr);
        // } else {
        //     //devolver error
        // }
        // let new_value = 0 - decr;
        // self.items.insert(key.to_string(), ValueTimeItem::new(ValueType::StringType(new_value.to_string()), KeyAccessTime::Volatile(3423423)));
        // Ok(new_value)

        // for item in self.items.iter_mut() {
        //     let k = item.get_key();
        //     if k == key {
        //         if let ValueType::StringType(str) = item.get_copy_of_value() {
        //             let str_as_number = str.parse::<i64>()?;
        //             let new_value = ValueType::StringType((str_as_number - decr).to_string());
        //             item.set_value(new_value);
        //             return Ok(str_as_number - decr);
        //         } else {
        //             //devolver error
        //         }
        //     }
        // }
        // let new_value = 0 - decr;
        // self.items.push(KeyValueItem::new(
        //     key.to_string(),
        //     ValueType::StringType(new_value.to_string()),
        // ));
        // Ok(new_value)
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
        // for item in self.items.iter_mut() {
        //     let k = item.get_key();
        //     if k == key {
        //         if let ValueType::StringType(str) = item.get_copy_of_value() {
        //             let str_as_number = str.parse::<i64>()?;
        //             let new_value = ValueType::StringType((str_as_number + incr).to_string());
        //             item.set_value(new_value);
        //             return Ok(str_as_number + incr);
        //         } else {
        //             //devolver error
        //         }
        //     }
        // }
        // let new_value = incr;
        // self.items.push(KeyValueItem::new(
        //     key.to_string(),
        //     ValueType::StringType(new_value.to_string()),
        // ));
        // Ok(new_value)
    }

    /// Devuelve la clave si el valor asociado es un string
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
    /// Devuelve la clave si el valor asociado es un string, sino devuelve nil
    pub fn get_value_by_key_or_nil(&self, key: &str) -> Option<String> {
        let item = self.items.get(&key.to_string());
        if let Some(item) = item {
            let value = item.get_copy_of_value();
            if let ValueType::StringType(str) = value {
                Some(str)
            } else {
                Some("(nil)".to_string())
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
                    let kvis = KeyValueItemSerialized::_new(kvi_serialized);
                    let kvis = kvis.transform_to_item();
                    self.items.insert(kvis.0, kvis.1);
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

    pub fn _save_items_to_file(&self) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create_new(false)
            .open(self.dbfilename.to_string())
            .unwrap();

        for kvi in &self.items {
            let kvi_type = match kvi.1._get_value() {
                ValueType::StringType(_) => "string",
                ValueType::SetType(_) => "set",
                ValueType::ListType(_) => "list",
            };
            writeln!(
                file,
                "{};{};{};{}",
                kvi.0,
                kvi.1._get_last_access_time().to_string(),
                kvi_type,
                kvi.1._get_value().to_string()
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

    // pub fn _add(&mut self, key: String, vt_item: ValueTimeItem) {//kv_item: KeyValueItem) {
    //     self.items.insert(kv_item.key, vt_item);
    // }

    // pub fn _delete_by_index(&mut self, index: usize) {
    //     self.items.remove(index);
    // }

    // pub fn add(&mut self, kv_item: KeyValueItem) {
    //     self.items.push(kv_item);
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::key_value_item::{KeyAccessTime, ValueType};

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
        std::fs::remove_file("./src/dummy.txt").unwrap();
    }

    // #[test]
    // fn empty_database_returns_cero() {
    //     let db = Database {
    //         dbfilename: "file".to_string(),
    //         items: vec![],
    //     };

    //     assert_eq!(db.get_size(), 0);
    // }

    //------------------------

    // #[test]
    // fn test_01_database_copies_value_to_new_key() {
    //     let mut db = Database::new(String::from("./src/dummy.txt"));

    //     let source = String::from("clave_1");
    //     let destination = String::from("clone");
    //     assert_eq!(db.copy(source, destination, false).unwrap(), ());

    //     let new_item = db.search_item_by_key(String::from("clone")).unwrap();
    //     if let ValueType::StringType(str) = new_item._get_value() {
    //         assert_eq!(str, &String::from("valor_1"));
    //     }
    // }

    // #[test]
    // fn test_02_database_copy_replaces_key_with_new_value() {
    //     let mut db = Database::new(String::from("./src/dummy2.txt"));

    //     let source = String::from("clave_1");
    //     let destination = String::from("clone");
    //     assert_eq!(db.copy(source, destination, false).unwrap(), ());

    //     let new_item = db.search_item_by_key(String::from("clone")).unwrap();
    //     if let ValueType::StringType(str) = new_item._get_value() {
    //         assert_eq!(str, &String::from("valor_1"));
    //     }

    //     let source = String::from("clave_2");
    //     let destination = String::from("clone");
    //     assert_eq!(db.copy(source, destination, true).unwrap(), ());

    //     let new_item = db.search_item_by_key(String::from("clone")).unwrap();
    //     if let ValueType::StringType(str) = new_item._get_value() {
    //         assert_eq!(str, &String::from("valor_2"));
    //     }
    // }

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
        match *item._get_last_access_time() {
            KeyAccessTime::Persistent => assert!(true),
            KeyAccessTime::Volatile(_tmt) => assert!(false),
        }

        std::fs::remove_file("file").unwrap();
    }

    // use crate::domain::entities::key_value_item::ValueType;
    // use std::collections::LinkedList;
    // use std::io::{BufReader, Write};

    // #[test]
    // fn empty_database_returns_cero() {
    //     let db = Database {
    //         dbfilename: "file".to_string(),
    //         items: vec![],
    //     };

    //     assert_eq!(db.get_size(), 0);
    // }

    // #[test]
    // fn size_in_memory_is_correct() {
    //     let kv_item = KeyValueItem::new(
    //         String::from("123"),
    //         ValueType::StringType(String::from("222")),
    //     );
    //     let kv_item2 = KeyValueItem::new(
    //         String::from("123"),
    //         ValueType::StringType(String::from("222")),
    //     );

    //     let db = Database {
    //         dbfilename: "file".to_string(),
    //         items: vec![kv_item, kv_item2],
    //     };

    //     assert_eq!(db.get_size(), 2);
    // }

    // #[test]
    // fn add_item() {
    //     let added_item = KeyValueItem::new(
    //         String::from("nueva_key"),
    //         ValueType::StringType(String::from("222")),
    //     );
    //     let mut db = Database {
    //         dbfilename: "file".to_string(),
    //         items: vec![],
    //     };
    //     db.add(added_item);

    //     assert_eq!(db.items.first().unwrap().key, String::from("nueva_key"));
    //     assert_eq!(
    //         db.items.first().unwrap().value.to_string(),
    //         String::from("222")
    //     );
    //     assert_eq!(db.items.len(), 1)
    // }

    // #[test]
    // fn delete_item() {
    //     let added_item = KeyValueItem::new(
    //         String::from("nueva_key"),
    //         ValueType::StringType(String::from("222")),
    //     );
    //     let mut db = Database {
    //         dbfilename: "file".to_string(),
    //         items: vec![added_item],
    //     };
    //     assert_eq!(db.items.len(), 1);
    //     db._delete_by_index(0);
    //     assert_eq!(db.items.len(), 0);
    // }

    // #[test]
    // fn filename_is_correct() {
    //     let db = Database {
    //         dbfilename: "file".to_string(),
    //         items: vec![],
    //     };
    //     assert_eq!(db._get_filename(), "file".to_string());
    // }

    // #[test]
    // fn load_items_from_file() {
    //     let mut file = File::create("file_5".to_string()).expect("Unable to open");
    //     file.write_all(b"123key;;string;value\n").unwrap();
    //     file.write_all(b"124key;1623433677;string;value2\n")
    //         .unwrap();

    //     let db = Database::new("file_5".to_string());
    //     assert_eq!(db.items.len(), 2);
    //     let mut iter = db.items.iter();
    //     let kvi = iter.next().unwrap();

    //     assert_eq!(kvi.key.to_owned(), "123key");
    //     assert_eq!(kvi.value.to_string(), String::from("value"));
    //     match kvi.last_access_time {
    //         KeyAccessTime::Persistent => assert!(true),
    //         KeyAccessTime::Volatile(_) => assert!(false),
    //     }

    //     let kvi2 = iter.next().unwrap();
    //     assert_eq!(kvi2.key.to_owned(), "124key");
    //     assert_eq!(kvi2.value.to_string(), String::from("value2"));
    //     match kvi2.last_access_time {
    //         KeyAccessTime::Volatile(1623433677) => assert!(true),
    //         _ => assert!(false),
    //     }
    //     std::fs::remove_file("file_5").unwrap();
    // }

    // #[test]
    // fn create_database_file() {
    //     assert!(!std::path::Path::new("new_file").exists());
    //     let _db = Database::new("new_file".to_string());
    //     assert!(std::path::Path::new("new_file").exists());
    //     std::fs::remove_file("new_file").unwrap();
    // }

    // #[test]
    // fn save_items_to_file() {
    //     let mut _file = File::create("file".to_string()).expect("Unable to open");

    //     let mut db = Database::new("file".to_string());
    //     db.add(KeyValueItem {
    //         key: "clave_1".to_string(),
    //         value: ValueType::StringType("valor_1".to_string()),
    //         last_access_time: KeyAccessTime::Persistent,
    //     });
    //     let mut un_list = LinkedList::new();
    //     un_list.push_back("un_item_string".to_string());
    //     un_list.push_back("segundo_item_list_string".to_string());

    //     db.add(KeyValueItem {
    //         key: "clave_2".to_string(),
    //         value: ValueType::ListType(un_list),
    //         last_access_time: KeyAccessTime::Volatile(1231230),
    //     });

    //     db._save_items_to_file();

    //     let file = File::open(&db.dbfilename);
    //     let reader = BufReader::new(file.unwrap());
    //     let mut it = reader.lines();
    //     match it.next().unwrap() {
    //         Ok(t) => assert_eq!(t, "clave_1;;string;valor_1"),
    //         _ => assert!(false),
    //     }
    //     match it.next().unwrap() {
    //         Ok(t) => assert_eq!(
    //             t,
    //             "clave_2;1231230;list;un_item_string,segundo_item_list_string"
    //         ),
    //         _ => assert!(false),
    //     }

    //     std::fs::remove_file("file").unwrap();
    // }

    // #[test]
    // fn test_01_database_copies_value_to_new_key() {
    //     let mut db = Database::new(String::from("./src/dummy_copy_1.txt"));
    //     db.add(KeyValueItem {
    //         key: "clave_1".to_string(),
    //         value: ValueType::StringType("valor_1".to_string()),
    //         last_access_time: KeyAccessTime::Persistent,
    //     });

    //     let source = String::from("clave_1");
    //     let destination = String::from("clone");
    //     assert_eq!(db.copy(source, destination, false).unwrap(), ());

    //     let new_item = db.search_item_by_key(&String::from("clone")).unwrap();
    //     if let ValueType::StringType(str) = new_item._get_value() {
    //         assert_eq!(str, &String::from("valor_1"));
    //     }
    //     std::fs::remove_file("./src/dummy_copy_1.txt").unwrap();
    // }

    // #[test]
    // fn test_02_database_copy_replaces_key_with_new_value() {
    //     let mut db = Database::new(String::from("./src/dummy_copy.txt"));
    //     db.add(KeyValueItem {
    //         key: "clave_1".to_string(),
    //         value: ValueType::StringType("valor_1".to_string()),
    //         last_access_time: KeyAccessTime::Persistent,
    //     });

    //     let source = String::from("clave_1");
    //     let destination = String::from("clone");
    //     assert_eq!(db.copy(source, destination, false).unwrap(), ());

    //     let new_item = db.search_item_by_key(&String::from("clone")).unwrap();
    //     if let ValueType::StringType(str) = new_item._get_value() {
    //         assert_eq!(str, &String::from("valor_1"));
    //     }
    //     std::fs::remove_file("./src/dummy_copy.txt").unwrap();
    //}
    //------------------------------

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

    // fn test_03_clean_items_deletes_all_items() {
    //     let mut db = Database::new(String::from("./src/database_1.txt"));
    //     db.add(KeyValueItem {
    //         key: "clave_1".to_string(),
    //         value: ValueType::StringType("value".to_string()),
    //         last_access_time: KeyAccessTime::Persistent,
    //     });
    //     db.add(KeyValueItem {
    //         key: "clave_1".to_string(),
    //         value: ValueType::StringType("value".to_string()),
    //         last_access_time: KeyAccessTime::Persistent,
    //     });
    //     assert_eq!(db.get_size(), 2);
    //     db.clean_items();
    //     assert_eq!(db.get_size(), 0);
    //     std::fs::remove_file("./src/database_1.txt").unwrap();
    // }

    // #[test]
    // fn test_02_deletes_an_item_succesfully() {
    //     //let _file = File::create("./src/database.txt");
    //     let mut db = Database::new(String::from("./src/database.txt"));
    //     db.add(KeyValueItem {
    //         key: "clave_1".to_string(),
    //         value: ValueType::StringType("value".to_string()),
    //         last_access_time: KeyAccessTime::Persistent,
    //     });

    //     println!("{:?}", db._get_items());
    //     db.delete_key("clave_1".to_string());
    //     println!("{:?}", db._get_items());
    //     assert_eq!(db.get_size(), 0);
    //     std::fs::remove_file("./src/database.txt".to_string()).unwrap();
    // }

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
        match *item._get_last_access_time() {
            KeyAccessTime::Persistent => assert!(true),
            KeyAccessTime::Volatile(_tmt) => assert!(false),
        }
        std::fs::remove_file("./src/dummy_persist.txt".to_string()).unwrap();
    }
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
    // let _file = File::create("./src/dummy_appends_1.txt");
    let mut db = Database::new(String::from("./src/dummy_appends_1.txt"));

    let len = db.append_string(&"mykey".to_string(), &" World".to_string());
    assert_eq!(len, 6);
    std::fs::remove_file("./src/dummy_appends_1.txt".to_string()).unwrap();
}

#[test]
fn test_12_decr_key_to_existing_key() {
    // let _file = File::create("./src/dummy_dec.txt");
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
    // let _file = File::create("./src/dummy_decr.txt");
    let mut db = Database::new(String::from("./src/dummy_decr.txt"));

    let res = db.decrement_key_by(&"mykey".to_string(), 3).unwrap();
    assert_eq!(res, -3);
    std::fs::remove_file("./src/dummy_decr.txt".to_string()).unwrap();
}

#[test]
fn test_14_decr_by_to_invalid_string_value() {
    // let _file = File::create("./src/dummy.txt");
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
    std::fs::remove_file("./src/dummy_decr_2.txt".to_string()).unwrap();
}

#[test]
fn test_15_se_obtienen_valores_de_claves_externas_a_partir_de_un_patron_y_una_lista_de_elementos() {
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
    println!("{:?}", algo)
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
}

#[test]
fn test_19_se_obtienen_keys_que_contienen_patron_regex_excepto_exp_entre_corchetes_tipo_2_rango() {
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
}

#[test]
fn test_21_se_obtienen_las_claves_que_contienen_solo_string_values() {
    use std::collections::HashSet;

    let mut db = Database::new("file10".to_string());

    let vt_1 = ValueTimeItem {
        value: ValueType::StringType("hola".to_string()),
        last_access_time: KeyAccessTime::Volatile(0),
    };
    let vt_2 = ValueTimeItem {
        value: ValueType::StringType("chau".to_string()),
        last_access_time: KeyAccessTime::Volatile(0),
    };
    let vt_3 = ValueTimeItem {
        value: ValueType::ListType(vec!["hola".to_string(), "chau".to_string()]),
        last_access_time: KeyAccessTime::Volatile(0),
    };
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());
    this_set.insert("value_2".to_string());
    let vt_4 = ValueTimeItem {
        value: ValueType::SetType(this_set),
        last_access_time: KeyAccessTime::Volatile(0),
    };
    db.items.insert("saludo".to_string(), vt_1);
    db.items.insert("despido".to_string(), vt_2);
    db.items.insert("saludo_despido".to_string(), vt_3);
    db.items.insert("valores".to_string(), vt_4);

    let aux = db.get_value_by_key_or_nil("saludo").unwrap();
    println!("{:?}", aux)
}

#[test]
fn test_22_no_se_obtiene_la_clave_porque_tiene_value_tipo_list() {
    use std::collections::HashSet;

    let mut db = Database::new("file10".to_string());

    let vt_1 = ValueTimeItem {
        value: ValueType::StringType("hola".to_string()),
        last_access_time: KeyAccessTime::Volatile(0),
    };
    let vt_2 = ValueTimeItem {
        value: ValueType::StringType("chau".to_string()),
        last_access_time: KeyAccessTime::Volatile(0),
    };
    let vt_3 = ValueTimeItem {
        value: ValueType::ListType(vec!["hola".to_string(), "chau".to_string()]),
        last_access_time: KeyAccessTime::Volatile(0),
    };
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());
    this_set.insert("value_2".to_string());
    let vt_4 = ValueTimeItem {
        value: ValueType::SetType(this_set),
        last_access_time: KeyAccessTime::Volatile(0),
    };
    db.items.insert("saludo".to_string(), vt_1);
    db.items.insert("despido".to_string(), vt_2);
    db.items.insert("saludo_despido".to_string(), vt_3);
    db.items.insert("valores".to_string(), vt_4);

    let aux = db.get_value_by_key_or_nil("saludo_despido").unwrap();
    println!("{:?}", aux)
}
