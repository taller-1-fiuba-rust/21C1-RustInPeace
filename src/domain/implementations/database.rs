// use std::error::Error;
// use std::u64;
use std::collections::HashMap; //, ops::Bound};

//use regex::{Captures, Regex};

//use crate::domain::entities::key_value_item::ValueType;
use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};

#[derive(Debug)]
pub struct Database {
    dbfilename: String,
    items: HashMap<String, ValueTimeItem>,
    // items: Vec<KeyValueItem>,
}

impl Database {
    pub fn new(filename: String) -> Database {
        //este hashmap lo creo de prueba
        let aux_hashmap = HashMap::new();
        Database {
            dbfilename: filename,
            items: aux_hashmap,
        }
    }
    pub fn _get_filename(&self) -> String {
        self.dbfilename.clone()
    }

    pub fn _get_items(&self) -> &HashMap<String, ValueTimeItem> {
        &self.items
    }
    pub fn clean_items(&mut self) -> &HashMap<String, ValueTimeItem> {
        self.items.clear();
        &self.items
    }

    pub fn search_item_by_key(&self, key: String) -> Option<&ValueTimeItem> {
        match self.items.get(&key) {
            Some(item) => return Some(item),
            None => None,
        }
    }
    pub fn _get_keys_that_match_pattern(&self, pattern: String) -> Vec<String> {
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

    pub fn get_values_of_external_keys_that_match_a_pattern(
        &self,
        elements: Vec<String>,
        pat: &String,
    ) -> Option<Vec<(String, String)>> {
        let mut vec_auxiliar = Vec::new();
        let elements = elements.clone();
        for element in elements {
            let patterned_key = pat.to_string() + element.as_str();
            if let Some(patterned_key_value) = self
                .search_item_by_key(patterned_key)
                .unwrap()
                .get_value_version_2()
            {
                let vectorcito = (element, patterned_key_value[0].to_string());
                vec_auxiliar.push(vectorcito);
            } else {
                //
            }
        }
        if !vec_auxiliar.is_empty() {
            Some(vec_auxiliar)
        } else {
            None
        }
    }

    // pub fn get_values_of_external_keys_that_match_a_pattern(&self , pat: &str) {
    //     let current_regex = Regex::new(pat).unwrap();
    //     for (clave, valor) in self.items.iter().filter(|x|
    //         current_regex.is_match(x.0)
    //     ) {
    //         println!("clave {}", clave);

    //     }

    //         //current_

    // }

    pub(crate) fn copy(
        &mut self,
        source: String,
        destination: String,
        replace: bool,
    ) -> Option<()> {
        let source_item = self.search_item_by_key(source);
        if let Some(source_item) = source_item {
            let new_value = source_item.get_copy_of_value();
            let new_value_time = ValueTimeItem::new(new_value);
            if self.items.contains_key(&destination) {
                if replace {
                    self.items.insert(destination, new_value_time);
                    return Some(());
                } else {
                    return None;
                }
            } else {
                self.items.entry(destination).or_insert(new_value_time);
                return Some(());
            }
        }
        None
    }

    pub fn persist(&mut self, key: String) -> bool {
        match self.items.get_mut(&key) {
            Some(item) => item.make_persistent(),
            None => false,
        }
    }

    pub fn rename_key(&mut self, current_key: String, new_key: String) -> bool {
        let item = self.items.remove(&current_key);
        if let Some(item) = item {
            self.items.insert(new_key, item);
            return true;
        } else {
            return false;
        }
    }

    /* Si el servidor se reinicia se deben cargar los items del file */
    /* TODO los comento para que clippy no se queje hasta q los implementemos
    pub fn load_items(&self) {
        unimplemented!()
    }

    pub fn save_items_to_file(&self) {
        unimplemented!()
    }
    */
    pub fn _get_all_values(&self) -> Box<Vec<ValueType>> {
        let mut all_values = Vec::new();
        let values_vector = &self.items;
        for (_key, value) in values_vector {
            let only_value = value.get_copy_of_value();
            all_values.push(only_value);
        }
        let all_values_heap = Box::new(all_values);
        all_values_heap
    }

    pub fn get_size(&self) -> usize {
        self.items.len()
    }

    pub fn delete_key(&mut self, key: String) -> bool {
        if let Some(_key) = self.items.remove(&key) {
            true
        } else {
            false
        }
    }

    pub fn _add(&mut self, key: String, vt_item: ValueTimeItem) {
        self.items.insert(key, vt_item);
    }
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
        db._add("weight_bananas".to_string(), vt_1);
        db._add("apples_weight".to_string(), vt_2);
        db._add("deliciosos_kiwi_weight_baratos".to_string(), vt_3);
        db._add("banana_weight".to_string(), vt_4);

        // db.get_values_of_external_keys_that_match_a_pattern("banana");

        let vec_filtered = db._get_keys_that_match_pattern("weight".to_string());
        assert_eq!(vec_filtered.len(), 4)
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
    }

    #[test]
    fn test_04_deletes_an_item_succesfully() {
        let mut db = Database::new("file".to_string());

        let vt_1 = ValueTimeItem {
            value: ValueType::StringType("valor_1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_2 = ValueTimeItem {
            value: ValueType::StringType("valor_2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        db._add("weight_bananas".to_string(), vt_1);
        db._add("apples_weight".to_string(), vt_2);

        db.delete_key("apples_weight".to_string());

        assert_eq!(db.get_size(), 1)
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
        db._add("weight_bananas".to_string(), vt_1);
        db._add("apples_weight".to_string(), vt_2);
        //--------
        let _res = db.persist("weight_bananas".to_string());

        let item = db.search_item_by_key("weight_bananas".to_string()).unwrap();
        match *item._get_last_access_time() {
            KeyAccessTime::Persistent => assert!(true),
            KeyAccessTime::Volatile(_tmt) => assert!(false),
        }
    }
    //------------------------------

    #[test]
    fn test_08_size_in_memory_is_correct() {
        let mut db = Database::new("file".to_string());

        let vt_1 = ValueTimeItem {
            value: ValueType::StringType("valor_1".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let vt_2 = ValueTimeItem {
            value: ValueType::StringType("valor_2".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        db._add("weight_bananas".to_string(), vt_1);
        db._add("apples_weight".to_string(), vt_2);

        assert_eq!(db.get_size(), 2);
    }
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
    //     db.delete_by_index(0);
    //     assert_eq!(db.items.len(), 0);
    // }

    // #[test]
    // fn filename_is_correct() {
    //     let db = Database {
    //         dbfilename: "file".to_string(),
    //         items: vec![],
    //     };
    //     assert_eq!(db.get_filename(), "file".to_string());
    // }
}
