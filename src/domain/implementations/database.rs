use crate::domain::entities::key_value_item::KeyValueItem;
use crate::domain::entities::key_value_item_serialized::KeyValueItemSerialized;
use std::fs::File;
use std::io;
use std::io::BufRead;
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

    /* Si el servidor se reinicia se deben cargar los items del file */
    pub fn _load_items(&mut self) {
        if let Ok(lines) = Database::read_lines(self.dbfilename.to_string()) {
            for line in lines {
                if let Ok(kvi_serialized) = line {
                    let kvis = KeyValueItemSerialized::_new(kvi_serialized);
                    self.add(kvis.transform_to_item())
                }
            }
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

        assert_eq!(db._get_size(), 0);
    }
    #[test]
    fn size_in_memory_is_correct() {
        let kv_item = KeyValueItem::_new(
            String::from("123"),
            ValueType::StringType(String::from("222")),
        );
        let kv_item2 = KeyValueItem::_new(
            String::from("123"),
            ValueType::StringType(String::from("222")),
        );

        let db = Database {
            dbfilename: "file".to_string(),
            items: vec![kv_item, kv_item2],
        };

        assert_eq!(db._get_size(), 2);
    }
    #[test]
    fn add_item() {
        let added_item = KeyValueItem::_new(
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
        let added_item = KeyValueItem::_new(
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
    // #[test]
    /* fn load_items_from_file() {
        //TODO me falta mockear el archivo para que pueda correr el test.
        let db = Database::new("file".to_string());
        assert_eq!(db.items.len(), 3);
        assert_eq!(
            db.items.get(0).unwrap().value.to_string(),
            ValueType::StringType(String::from("222")).to_string()
        );
    }*/
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
        db.delete_by_index(0);
        assert_eq!(db.items.len(), 0);
    }

    #[test]
    fn filename_is_correct() {
        let db = Database {
            dbfilename: "file".to_string(),
            items: vec![],
        };
        assert_eq!(db.get_filename(), "file".to_string());
    }
}