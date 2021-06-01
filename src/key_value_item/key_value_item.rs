pub enum ValueType {
    List,
    Set,
    String
}

pub struct KeyValueItem{
    key: String,
    value: T // Limitar solo a los tipos permitidos.
}

impl KeyValueItem{
    fn new(value :T){
       KeyValueItem {key: "123", value: T}
    }
}

