use crate::domain::entities::key_value_item::KeyValueItem;

#[derive(Debug)]
pub struct Database {
    dbfilename: String,
    items: Vec<KeyValueItem>,
}

impl Database {
    pub fn new(filename: String) -> Database {
        Database {
            dbfilename: filename,
            items: vec![], //TODO al crear este objeto debería cargar los items del file.
        }
    }
    pub fn get_filename(&self) -> String {
        self.dbfilename.clone()
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

    pub fn delete_by_index(&mut self, index: usize) {
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
