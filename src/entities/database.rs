use crate::key_value_item::key_value_item_domain::KeyValueItem;

#[derive(Debug)]
pub struct Database {
    dbfilename: String,
    items: Vec<KeyValueItem>,
}

impl Database {
    pub fn get_filename(&self) -> String{
        self.dbfilename.clone()
    }

    /* Si el servidor se reinicia se deben cargar los items del file */
    pub fn load_items(&self) {
        unimplemented!()
    }

    pub fn save_items_to_file(&self) {
        unimplemented!()
    }

    pub fn get_size(&self) -> usize {
        self.items.len()
    }

    pub fn get_all_by_key(&self, _key: String) -> Vec<KeyValueItem> {
        unimplemented!()
    }

    pub fn delete(&self) {
        //chequeo si existe la key
        // si no existe salgo con error
        // si  existe elimino el item de la lista
         unimplemented!()
    }

    pub fn update(&self) {
        //chequeo si existe la key
        // si no existe agrego el item a la lista
        // si existe salgo con error
        unimplemented!()
    }

    pub fn add(&self) {
        //chequeo si existe la key
        // si no existe agrego el item a la lista
        // si existe salgo con error

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_value_item::key_value_item_domain::ValueType;

    #[test]
    fn empty_database_returns_cero() {
        let db = Database {
            dbfilename: "file".to_string(),
            items: vec![]
        };

        assert_eq!(db.get_size(), 0);
    }

    #[test]
    fn size_in_memory_is_correct() {
        let kv_item = KeyValueItem::new(String::from("123"), ValueType::StringType(String::from("222")));
        let kv_item2 = KeyValueItem::new(String::from("123"), ValueType::StringType(String::from("222")));

        let db = Database {
            dbfilename: "file".to_string(),
            items: vec![kv_item,kv_item2]
        };

        assert_eq!(db.get_size(), 2);
    }
}
