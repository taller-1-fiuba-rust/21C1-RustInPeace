pub enum ValueType {
    List,
    Set,
    String
}

pub struct KeyValueItem{
    key: String,
    value: T ,// Limitar solo a los tipos permitidos.
    last_access_time: String //Cambiar por timestamp
}

impl KeyValueItem{
    fn new(value :T){
       KeyValueItem {key: "123", value: T, last_access_time: "now"}
    }
}

