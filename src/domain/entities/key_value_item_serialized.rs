use crate::domain::entities::key_value_item::{KeyValueItem, ValueType};

pub struct KeyValueItemSerialized{
    line: String
}

impl KeyValueItemSerialized{
    pub fn _new(line: String) -> KeyValueItemSerialized {
        KeyValueItemSerialized {
            line
        }
    }
    pub fn transform_to_item(&self) -> KeyValueItem{
        // Format: key, access_time, value
        KeyValueItem{
            key: "".to_string(),
            value: ValueType::StringType("".to_string()),
            last_access_time: 0
        }
    }
}