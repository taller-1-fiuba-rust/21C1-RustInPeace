#[allow(unused)]
use crate::domain::entities::key_value_item::KeyAccessTime;

#[allow(unused)]
use crate::domain::entities::key_value_item::{KeyValueItem, ValueType};
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
    items: Vec<KeyValueItem>,
}

impl Database {
    pub fn new(filename: String) -> Database {
        let mut db = Database {
            dbfilename: filename,
            items: vec![],
        };
        db.load_items();
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
                self.replace_value_on_key(destination, new_value)
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

    pub fn append_string(&mut self, key: &str, string: &str) -> usize {
        for item in self.items.iter_mut() {
            let k = item.get_key();
            if k == key {
                if let ValueType::StringType(old_value) = item.get_copy_of_value() {
                    let len = old_value.len() + string.len();
                    let new_value = ValueType::StringType(old_value + string);
                    item.set_value(new_value);
                    return len;
                }
            }
        }
        self.items.push(KeyValueItem::new(
            key.to_string(),
            ValueType::StringType(string.to_string()),
        ));
        string.len()
    }

    pub fn decrement_key_by(&mut self, key: &str, decr: i64) -> Result<i64, ParseIntError> {
        for item in self.items.iter_mut() {
            let k = item.get_key();
            if k == key {
                if let ValueType::StringType(str) = item.get_copy_of_value() {
                    let str_as_number = str.parse::<i64>()?;
                    let new_value = ValueType::StringType((str_as_number - decr).to_string());
                    item.set_value(new_value);
                    return Ok(str_as_number - decr);
                } else {
                    //devolver error
                }
            }
        }
        let new_value = 0 - decr;
        self.items.push(KeyValueItem::new(
            key.to_string(),
            ValueType::StringType(new_value.to_string()),
        ));
        Ok(new_value)
    }

    pub fn increment_key_by(&mut self, key: &str, incr: i64) -> Result<i64, ParseIntError> {
        for item in self.items.iter_mut() {
            let k = item.get_key();
            if k == key {
                if let ValueType::StringType(str) = item.get_copy_of_value() {
                    let str_as_number = str.parse::<i64>()?;
                    let new_value = ValueType::StringType((str_as_number + incr).to_string());
                    item.set_value(new_value);
                    return Ok(str_as_number + incr);
                } else {
                    //devolver error
                }
            }
        }
        let new_value = incr;
        self.items.push(KeyValueItem::new(
            key.to_string(),
            ValueType::StringType(new_value.to_string()),
        ));
        Ok(new_value)
    }

    //agregar tests
    pub fn get_value_by_key(&self, key: &str) -> Option<String> {
        let item = self.search_item_by_key(key);
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
        let item = self.search_item_by_key(key);
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
        let item = self.search_item_by_key(key);
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
        let item = self.search_item_by_key(key);
        if let Some(item) = item {
            let value = item.get_copy_of_value();
            if let ValueType::StringType(str) = value {
                self.replace_value_on_key(
                    key.to_string(),
                    ValueType::StringType(new_value.to_string()),
                );
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
                    self.add(kvis.transform_to_item())
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
            let kvi_type = match kvi.value {
                ValueType::StringType(_) => "string",
                ValueType::SetType(_) => "set",
                ValueType::ListType(_) => "list",
            };
            writeln!(
                file,
                "{};{};{};{}",
                kvi.key,
                kvi.last_access_time.to_string(),
                kvi_type,
                kvi.value.to_string()
            )
            .unwrap();
        }
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
    use std::collections::LinkedList;
    use std::io::{BufReader, Write};

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

    #[test]
    fn load_items_from_file() {
        let mut file = File::create("file_5".to_string()).expect("Unable to open");
        file.write_all(b"123key;;string;value\n").unwrap();
        file.write_all(b"124key;1623433677;string;value2\n")
            .unwrap();

        let db = Database::new("file_5".to_string());
        assert_eq!(db.items.len(), 2);
        let mut iter = db.items.iter();
        let kvi = iter.next().unwrap();

        assert_eq!(kvi.key.to_owned(), "123key");
        assert_eq!(kvi.value.to_string(), String::from("value"));
        match kvi.last_access_time {
            KeyAccessTime::Persistent => assert!(true),
            KeyAccessTime::Volatile(_) => assert!(false),
        }

        let kvi2 = iter.next().unwrap();
        assert_eq!(kvi2.key.to_owned(), "124key");
        assert_eq!(kvi2.value.to_string(), String::from("value2"));
        match kvi2.last_access_time {
            KeyAccessTime::Volatile(1623433677) => assert!(true),
            _ => assert!(false),
        }
        std::fs::remove_file("file_5").unwrap();
    }

    #[test]
    fn create_database_file() {
        assert!(!std::path::Path::new("new_file").exists());
        let _db = Database::new("new_file".to_string());
        assert!(std::path::Path::new("new_file").exists());
        std::fs::remove_file("new_file").unwrap();
    }

    #[test]
    fn save_items_to_file() {
        let mut _file = File::create("file".to_string()).expect("Unable to open");

        let mut db = Database::new("file".to_string());
        db.add(KeyValueItem {
            key: "clave_1".to_string(),
            value: ValueType::StringType("valor_1".to_string()),
            last_access_time: KeyAccessTime::Persistent,
        });
        let mut un_list = LinkedList::new();
        un_list.push_back("un_item_string".to_string());
        un_list.push_back("segundo_item_list_string".to_string());

        db.add(KeyValueItem {
            key: "clave_2".to_string(),
            value: ValueType::ListType(un_list),
            last_access_time: KeyAccessTime::Volatile(1231230),
        });

        db._save_items_to_file();

        let file = File::open(&db.dbfilename);
        let reader = BufReader::new(file.unwrap());
        let mut it = reader.lines();
        match it.next().unwrap() {
            Ok(t) => assert_eq!(t, "clave_1;;string;valor_1"),
            _ => assert!(false),
        }
        match it.next().unwrap() {
            Ok(t) => assert_eq!(
                t,
                "clave_2;1231230;list;un_item_string,segundo_item_list_string"
            ),
            _ => assert!(false),
        }

        std::fs::remove_file("file").unwrap();
    }

    #[test]
    fn test_01_database_copies_value_to_new_key() {
        let mut db = Database::new(String::from("./src/dummy_copy_1.txt"));
        db.add(KeyValueItem {
            key: "clave_1".to_string(),
            value: ValueType::StringType("valor_1".to_string()),
            last_access_time: KeyAccessTime::Persistent,
        });

        let source = String::from("clave_1");
        let destination = String::from("clone");
        assert_eq!(db.copy(source, destination, false).unwrap(), ());

        let new_item = db.search_item_by_key(&String::from("clone")).unwrap();
        if let ValueType::StringType(str) = new_item._get_value() {
            assert_eq!(str, &String::from("valor_1"));
        }
        std::fs::remove_file("./src/dummy_copy_1.txt").unwrap();
    }

    #[test]
    fn test_02_database_copy_replaces_key_with_new_value() {
        let mut db = Database::new(String::from("./src/dummy_copy.txt"));
        db.add(KeyValueItem {
            key: "clave_1".to_string(),
            value: ValueType::StringType("valor_1".to_string()),
            last_access_time: KeyAccessTime::Persistent,
        });

        let source = String::from("clave_1");
        let destination = String::from("clone");
        assert_eq!(db.copy(source, destination, false).unwrap(), ());

        let new_item = db.search_item_by_key(&String::from("clone")).unwrap();
        if let ValueType::StringType(str) = new_item._get_value() {
            assert_eq!(str, &String::from("valor_1"));
        }
        std::fs::remove_file("./src/dummy_copy.txt").unwrap();
    }

    #[test]
    fn test_03_clean_items_deletes_all_items() {
        let mut db = Database::new(String::from("./src/database_1.txt"));
        db.add(KeyValueItem {
            key: "clave_1".to_string(),
            value: ValueType::StringType("value".to_string()),
            last_access_time: KeyAccessTime::Persistent,
        });
        db.add(KeyValueItem {
            key: "clave_1".to_string(),
            value: ValueType::StringType("value".to_string()),
            last_access_time: KeyAccessTime::Persistent,
        });
        assert_eq!(db.get_size(), 2);
        db.clean_items();
        assert_eq!(db.get_size(), 0);
        std::fs::remove_file("./src/database_1.txt").unwrap();
    }

    #[test]
    fn test_02_deletes_an_item_succesfully() {
        //let _file = File::create("./src/database.txt");
        let mut db = Database::new(String::from("./src/database.txt"));
        db.add(KeyValueItem {
            key: "clave_1".to_string(),
            value: ValueType::StringType("value".to_string()),
            last_access_time: KeyAccessTime::Persistent,
        });

        println!("{:?}", db._get_items());
        db.delete_key("clave_1".to_string());
        println!("{:?}", db._get_items());
        assert_eq!(db.get_size(), 0);
        std::fs::remove_file("./src/database.txt".to_string()).unwrap();
    }

    #[test]
    fn persist_changes_type_of_access_time() {
        use crate::domain::entities::key_value_item::KeyAccessTime;
        // let _file = File::create("./src/dummy.txt");
        let mut db = Database::new(String::from("./src/dummy_persist.txt"));
        let _res = db.add(KeyValueItem {
            key: "clave_1".to_string(),
            value: ValueType::StringType("value".to_string()),
            last_access_time: KeyAccessTime::Persistent,
        });

        let item = db.search_item_by_key("clave_1").unwrap();
        match *item._get_last_access_time() {
            KeyAccessTime::Persistent => assert!(true),
            KeyAccessTime::Volatile(_tmt) => assert!(false),
        }
        std::fs::remove_file("./src/dummy_persist.txt".to_string()).unwrap();
    }
    // std::fs::remove_file("./src/dummy.txt".to_string()).unwrap();
}

#[test]
fn append_adds_string_to_end_of_existing_value() {
    // let _file = File::create("./src/dummy.txt");
    let mut db = Database::new(String::from("./src/dummy_appends_2.txt"));
    let _res = db.add(KeyValueItem {
        key: "mykey".to_string(),
        value: ValueType::StringType("Hello".to_string()),
        last_access_time: KeyAccessTime::Persistent,
    });

    let len = db.append_string(&"mykey".to_string(), &" World".to_string());
    assert_eq!(len, 11);
    std::fs::remove_file("./src/dummy_appends_2.txt".to_string()).unwrap();
}

#[test]
fn append_adds_string_to_new_value() {
    // let _file = File::create("./src/dummy_appends_1.txt");
    let mut db = Database::new(String::from("./src/dummy_appends_1.txt"));

    let len = db.append_string(&"mykey".to_string(), &" World".to_string());
    assert_eq!(len, 6);
    std::fs::remove_file("./src/dummy_appends_1.txt".to_string()).unwrap();
}

#[test]
fn decr_key_to_existing_key() {
    // let _file = File::create("./src/dummy_dec.txt");
    let mut db = Database::new(String::from("./src/dummy_decr_1.txt"));
    let _res = db.add(KeyValueItem {
        key: "mykey".to_string(),
        value: ValueType::StringType("10".to_string()),
        last_access_time: KeyAccessTime::Persistent,
    });

    let res = db.decrement_key_by(&"mykey".to_string(), 3).unwrap();
    assert_eq!(res, 7);
    std::fs::remove_file("./src/dummy_decr_1.txt".to_string()).unwrap();
}

#[test]
fn decr_by_to_new_key() {
    // let _file = File::create("./src/dummy_decr.txt");
    let mut db = Database::new(String::from("./src/dummy_decr.txt"));

    let res = db.decrement_key_by(&"mykey".to_string(), 3).unwrap();
    assert_eq!(res, -3);
    std::fs::remove_file("./src/dummy_decr.txt".to_string()).unwrap();
}

#[test]
fn decr_by_to_invalid_string_value() {
    // let _file = File::create("./src/dummy.txt");
    let mut db = Database::new(String::from("./src/dummy_decr_2.txt"));
    let _res = db.add(KeyValueItem {
        key: "mykey".to_string(),
        value: ValueType::StringType("Hello".to_string()),
        last_access_time: KeyAccessTime::Persistent,
    });

    let res = db.decrement_key_by(&"mykey".to_string(), 3);
    assert!(res.is_err());
    std::fs::remove_file("./src/dummy_decr_2.txt".to_string()).unwrap();
}
