use crate::key_value_item::key_value_item::KeyValueItem;

pub struct Database {
    config: String,
    items: Vec<KeyValueItem>,
}

impl Database {
    pub fn get_config(&self) {
        unimplemented!()
    }
    /* Si el servidor se reinicia se deben cargar los items del file */
    pub fn load_items(&self) {
        unimplemented!()
    }

    pub fn save_items_to_file(&self) {
        unimplemented!()
    }

    pub fn get_size(&self) -> String {
        self.items.len().to_string()
    }

    pub fn get_all_by_key(&self, key: String) -> Vec<KeyValueItem> {
        unimplemented!()
    }

    pub fn delete(&self) {
        unimplemented!()
    }

    pub fn update(&self) {
        unimplemented!()
    }

    pub fn create(&self) {
        unimplemented!()
    }
}
