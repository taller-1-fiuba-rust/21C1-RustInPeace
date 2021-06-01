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
    fn new(value :T) -> KeyValueItem {
       KeyValueItem {
           key: "123".to_string(),
           value: T,
           last_access_time: "now".to_string()
       }
    }
}

