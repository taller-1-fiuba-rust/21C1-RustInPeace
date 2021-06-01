use crate::entities::database::Database;
use crate::key_value_item::key_value_item::KeyValueItem;

pub struct KeyValueItemRepository {
    db: Database,
}

impl KeyValueItemRepository {
    fn new(connection: Database) -> KeyValueItemRepository {
        KeyValueItemRepository { db: connection }
    }
    pub fn delete_key(&self, key: String) -> Result<(), ()> {
        Ok(self.db.delete())
    }
    pub fn get_all() -> Result<KeyValueItem, ()> {
        unimplemented!()
    }

    pub fn get_by_key_and_type(&self, key: String, value_type: String) {
        unimplemented!()
        /* self.db
         .get_all_by_key(key)
        //Filtrar por type
        */
    }
}
