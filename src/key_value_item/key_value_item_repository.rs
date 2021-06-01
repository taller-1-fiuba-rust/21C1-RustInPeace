pub fn save() -> Result<(),()> {
    unimplemented!()
}
pub fn create() -> Result<(),()> {
    unimplemented!()
}
pub fn update() -> Result<(),()> {
    unimplemented!()
}
pub fn delete() -> Result<(),()> {
    unimplemented!()
}
pub fn get_all() -> Result< KeyValueItem,()> {
    unimplemented!()
}

pub fn get_by_key_and_type(key: String, valueType: KeyValueType){
    database
        .get_all_by_key(key)
        .filter(valueType)
}

