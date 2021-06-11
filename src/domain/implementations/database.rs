#[allow(unused)]
use crate::domain::entities::key_value_item::{KeyValueItem, ValueType};
use crate::domain::entities::key_value_item_serialized::KeyValueItemSerialized;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::BufRead;
use std::path::Path;
use crate::domain::entities::key_value_item::KeyAccessTime;
// use std::error::Error;
// use std::u64;

#[derive(Debug)]
pub struct Database {
    dbfilename: String,
    items: Vec<KeyValueItem>,
}

impl Database {
    pub fn new(filename: String) -> Database {
        let mut db = Database {
            dbfilename: filename,
            items: vec![],
        };
        db._load_items();
        db
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

    pub fn rename_key(&mut self, actual_key: String, new_key: String) {
        if let Some(pos) = self
            .items
            .iter()
            .position(|x| *x.get_key().to_string() == actual_key)
        {
            let saved_value = self.items.get(pos).unwrap().get_copy_of_value();
            self.items.remove(pos);
            let updated_key = KeyValueItem::new(new_key, saved_value);
            self.items.push(updated_key);
        }
    }

    /* Si el servidor se reinicia se deben cargar los items del file */
    pub fn _load_items(&mut self) {
        if let Ok(lines) = Database::read_lines(self.dbfilename.to_string()) {
            for line in lines {
                if let Ok(kvi_serialized) = line {
                    let kvis = KeyValueItemSerialized::_new(kvi_serialized);
                    self.add(kvis.transform_to_item())
                }
            }
        } else {
            panic!("Error al leer el archivo dump");
        }
    }

    //TODO sacar esto de acá
    pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(filename)?;
        Ok(io::BufReader::new(file).lines())
    }

    pub fn _save_items_to_file(&self) {
        unimplemented!()
    }

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

    pub fn add(&mut self, kv_item: KeyValueItem) {
        self.items.push(kv_item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::key_value_item::ValueType;
    #[test]
    fn empty_database_returns_cero() {
        let db = Database {
            dbfilename: "file".to_string(),
            items: vec![],
        };

        assert_eq!(db.get_size(), 0);
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
    #[test]
    fn add_item() {
        let added_item = KeyValueItem::new(
            String::from("nueva_key"),
            ValueType::StringType(String::from("222")),
        );
        let mut db = Database {
            dbfilename: "file".to_string(),
            items: vec![],
        };
        db.add(added_item);

        assert_eq!(db.items.first().unwrap().key, String::from("nueva_key"));
        assert_eq!(
            db.items.first().unwrap().value.to_string(),
            String::from("222")
        );
        assert_eq!(db.items.len(), 1)
    }
    #[test]
    fn delete_item() {
        let added_item = KeyValueItem::new(
            String::from("nueva_key"),
            ValueType::StringType(String::from("222")),
        );
        let mut db = Database {
            dbfilename: "file".to_string(),
            items: vec![added_item],
        };
        assert_eq!(db.items.len(), 1);
        db._delete_by_index(0);
        assert_eq!(db.items.len(), 0);
    }
    #[test]
    fn filename_is_correct() {
        let db = Database {
            dbfilename: "file".to_string(),
            items: vec![],
        };
        assert_eq!(db._get_filename(), "file".to_string());
    }
    /*
     #[test]

     fn load_items_from_file() {
        //TODO me falta mockear el archivo para que pueda correr el test.
        let db = Database::new("file".to_string());
        assert_eq!(db.items.len(), 4);
        assert_eq!(
            db.items.get(0).unwrap().value.to_string(),
            ValueType::StringType(String::from("222")).to_string()
        );
    }*/
}

/* TODO LO COMENTO PORQUE VAMOS A CAMBIAR ESOT.
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
}*/

#[test]
fn test_02_deletes_an_item_succesfully() {
    let _file = File::create("./src/database.txt");
    let mut db = Database::new(String::from("./src/database.txt"));
    db.add(KeyValueItem{key: "clave_1".to_string(),
        value: ValueType::StringType("value".to_string()),
        last_access_time: KeyAccessTime::Persistent });

    println!("{:?}", db._get_items());
    db.delete_key("clave_1".to_string());
    println!("{:?}", db._get_items());
    assert_eq!(db.get_size(), 0)
}

#[test]
fn persist_changes_type_of_access_time() {
    use crate::domain::entities::key_value_item::KeyAccessTime;
    let file = File::create("./src/dummy.txt");
    let mut db = Database::new(String::from("./src/dummy.txt"));
    let _res = db.add(KeyValueItem{
        key:"clave_1".to_string(),
        value: ValueType::StringType("value".to_string()),
        last_access_time: KeyAccessTime::Persistent
    });

    let item = db.search_item_by_key("clave_1").unwrap();
    match *item._get_last_access_time() {
        KeyAccessTime::Persistent => assert!(true),
        KeyAccessTime::Volatile(_tmt) => assert!(false),
    }
}
