use crate::domain::entities::key_value_item::{KeyValueItem, ValueType};
use std::collections::{HashSet, LinkedList};

// Format: key, access_time, type, value
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
        // Format: key, access_time, type, value
        let line: Vec<&str> = self.line.split(",").collect();
        let value = match line[2] {
            "string" => ValueType::StringType(line[3].to_string()),
            "set" => {
                let mut hashSet = HashSet::new();
                hashSet.insert(line[3].to_string());
                ValueType::SetType(hashSet)
            },
            "list" => {
                let mut list = LinkedList::new();
                list.push_back(line[3].to_string());
                ValueType::ListType(hashSet)
            },
            _ => { Err("Archivo corrupto. No pertenece a ningún tipo de dato soportado.")}
        };

        KeyValueItem {
            key: line[0].to_string(),
            value,
            last_access_time: line[1].parse::<u64>()?
        }
    }
}