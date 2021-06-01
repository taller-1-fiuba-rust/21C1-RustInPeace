pub enum ValueType {
    List,
    Set,
    String
}

pub struct KeyValueItem{
    key: String,
    value: String,// Extender a los tipos permitidos.
    last_access_time: String //Cambiar por timestamp
}

impl KeyValueItem{
    fn new(value :String) -> KeyValueItem {
       KeyValueItem {
           key: "123".to_string(),
           value,
           last_access_time: "now".to_string()
       }
    }
}

