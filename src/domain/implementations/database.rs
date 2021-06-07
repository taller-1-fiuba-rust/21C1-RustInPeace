// use std::error::Error;
// use std::u64;

use crate::domain::entities::key_value_item::KeyValueItem;
use crate::domain::entities::key_value_item::ValueType;

#[derive(Debug)]
pub struct Database {
    dbfilename: String,
    items: Vec<KeyValueItem>,
}

impl Database {
    pub fn new(filename: String) -> Database {
        Database {
            dbfilename: filename,
            //try_1 = KeyValueItem::new("clave_1".to_string(), StringType("valor_1".to_string()));
            items: vec![
                KeyValueItem::new(
                    "clave_1".to_string(),
                    ValueType::StringType("valor_1".to_string()),
                ),
                KeyValueItem::new(
                    "clave_2".to_string(),
                    ValueType::StringType("valor_2".to_string()),
                ),
            ], //TODO al crear este objeto debería cargar los items del file.
        }
    }
    pub fn _get_filename(&self) -> String {
        self.dbfilename.clone()
    }

    pub fn _get_items(&self) -> &Vec<KeyValueItem> {
        &self.items
    }
    pub fn clean_items(&mut self) -> &Vec<KeyValueItem> {
        self.items = Vec::new();
        &self.items
    }

    pub fn search_item_by_key(&self, key: &str) -> Option<&KeyValueItem> {
        for item in &self.items {
            let k = item.get_key();
            if k == key {
                return Some(item);
            }
        }
        None
    }

    pub fn copy(&mut self, source: String, destination: String, replace: bool) -> Option<()> {
        let source_item = self.search_item_by_key(&source);
        if let Some(source_item) = source_item {
            let new_value = source_item.get_copy_of_value();
            if replace {
                if let Some(()) = self.replace_value_on_key(destination, new_value) {
                    Some(())
                } else {
                    None
                }
            } else {
                let new_item = KeyValueItem::new(destination, new_value);
                self.items.push(new_item);
                Some(())
            }
        } else {
            None
        }
    }

    pub fn replace_value_on_key(&mut self, key: String, value: ValueType) -> Option<()> {
        for item in self.items.iter_mut() {
            let k = item.get_key();
            if k == &key {
                item.set_value(value);
                return Some(());
            }
        }
        None
    }

    pub fn persist(&mut self, key: String) -> bool {
        for item in self.items.iter_mut() {
            let k = item.get_key();
            if k == &key {
                return item.make_persistent();
            }
        }
        false
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
    pub fn get_size(&self) -> usize {
        self.items.len()
    }

    pub fn delete_key(&mut self, key: String) {
        if let Some(pos) = self
            .items
            .iter()
            .position(|x| *x.get_key().to_string() == key)
        {
            self.items.remove(pos);
        }
    }

    pub fn _delete_by_index(&mut self, index: usize) {
        self.items.remove(index);
    }

    pub fn _add(&mut self, kv_item: KeyValueItem) {
        self.items.push(kv_item);
    }
}

#[test]
fn test_01_database_copies_value_to_new_key() {
    let mut db = Database::new(String::from("./src/dummy.txt"));

    let source = String::from("clave_1");
    let destination = String::from("clone");
    assert_eq!(db.copy(source, destination, false).unwrap(), ());

    let new_item = db.search_item_by_key(&String::from("clone")).unwrap();
    if let ValueType::StringType(str) = new_item._get_value() {
        assert_eq!(str, &String::from("valor_1"));
    }
}

#[test]
fn test_02_database_copy_replaces_key_with_new_value() {
    let mut db = Database::new(String::from("./src/dummy.txt"));

    let source = String::from("clave_1");
    let destination = String::from("clone");
    assert_eq!(db.copy(source, destination, false).unwrap(), ());

    let new_item = db.search_item_by_key(&String::from("clone")).unwrap();
    if let ValueType::StringType(str) = new_item._get_value() {
        assert_eq!(str, &String::from("valor_1"));
    }

    let source = String::from("clave_2");
    let destination = String::from("clone");
    assert_eq!(db.copy(source, destination, true).unwrap(), ());

    let new_item = db.search_item_by_key(&String::from("clone")).unwrap();
    if let ValueType::StringType(str) = new_item._get_value() {
        assert_eq!(str, &String::from("valor_2"));
    }
}

#[test]
fn test_03_clean_items_deletes_all_items() {
    let mut db = Database::new(String::from("./src/database.txt"));
    db.clean_items();
    assert_eq!(db.get_size(), 0);
}

#[test]
fn test_02_deletes_an_item_succesfully() {
    let mut db = Database::new(String::from("./src/database.txt"));
    println!("{:?}", db._get_items());
    db.delete_key("clave_1".to_string());
    println!("{:?}", db._get_items());
    assert_eq!(db.get_size(), 1)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::key_value_item::ValueType;

    // #[test]
    // fn empty_database_returns_cero() {
    //     let db = Database {
    //         dbfilename: "file".to_string(),
    //         items: vec![],
    //     };

    //     assert_eq!(db.get_size(), 0);
    // }

    #[test]
    fn database_with_two_elements_returns_2() {
        let db = Database::new("filename".to_string());
        assert_eq!(db.get_size(), 2);
    }

    #[test]
    fn size_in_memory_is_correct() {
        let kv_item = KeyValueItem::new(
            String::from("123"),
            ValueType::StringType(String::from("222")),
        );
        let kv_item2 = KeyValueItem::new(
            String::from("123"),
            ValueType::StringType(String::from("222")),
        );

        let db = Database {
            dbfilename: "file".to_string(),
            items: vec![kv_item, kv_item2],
        };

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
