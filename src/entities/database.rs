use crate::key_value_item::key_value_item::KeyValueItem;

pub struct Database {
    config: String,
    items: Vec<KeyValueItem>, // Es probable que este vector almacene structs.
}

impl Database {
    pub fn get_config() {
        unimplemented!()
    }
    /* Si el servidor se reinicia se deben cargar los items del file */
    pub fn load_items() {
        unimplemented!()
    }

    pub fn save_items_to_file() {
        unimplemented!()
    }

    pub fn get_size(&self) -> String {
        self.items.len().to_string()
    }

    pub fn get_all_by_key(key: String) {
        unimplemented!()
    }

    pub fn delete() {
        unimplemented!()
    }

    pub fn update() {
        unimplemented!()
    }

    pub fn create() {
        unimplemented!()
    }
}