use crate::domain::implementations::database::Database;

#[derive(Debug)]
pub struct KeyValueItemRepository {
    db: Database,
}

impl KeyValueItemRepository {
    /* TODO DEJO COMENTADO HASTA Q LO IMPLEMENTEMOS

    pub fn new(connection: Database) -> KeyValueItemRepository {
    KeyValueItemRepository { db: connection }
    }
    pub fn delete_key(&self, _key: String) -> Result<(), ()> {
    Ok(self.db.delete())
    }
    pub fn get_all() -> Result<KeyValueItem, ()> {
    unimplemented!()
    }

    pub fn get_by_key_and_type(&self, _key: String, _value_type: String) {
    unimplemented!()
    /* self.db
     .get_all_by_key(key)
    //Filtrar por type
    */
    }
    pub fn update(&self) {
    //chequeo si existe la key
    // si no existe agrego el item a la lista
    // si existe salgo con error
    unimplemented!()
    }

    pub fn delete(){
    //chequeo si existe la key
    // si no existe salgo con error
    // si  existe elimino el item de la lista
    unimplemented!()
    }
    pub fn get_all_by_key(&self, _key: String) -> Vec<KeyValueItem> {
    unimplemented!()
    }


    pub fn create(){
    //chequeo si existe la key
    // si no existe agrego el item a la lista
    // si existe salgo con error
    }*/
}

#[cfg(test)]
mod tests {}