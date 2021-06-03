use std::collections::{HashSet, LinkedList};
use std::fmt;

#[derive(Debug)]
pub enum ValueType {
    ListType(LinkedList<String>),
    SetType(HashSet<String>),
    StringType(String),
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match self {
            ValueType::ListType(value) => {
                let mut printable_v = "".to_owned();
                for v in value {
                    printable_v.push_str(v);
                    printable_v.push(',')
                }
                printable_v.pop();
                printable_v
            }
            ValueType::SetType(value) => {
                let mut printable_v = "".to_owned();
                for v in value {
                    printable_v.push_str(v);
                    printable_v.push(',')
                }
                printable_v.pop();
                printable_v
            }
            ValueType::StringType(value) => value.to_string(),
        };
        write!(f, "{}", printable)
    }
}

#[derive(Debug)]
pub struct KeyValueItem {
    pub(crate) key: String, //TODO tuve que hacer publicos estos atributos porque los necesito para testear
    pub(crate) value: ValueType,
    pub(crate) last_access_time: u64,
}

impl KeyValueItem {
    pub fn new(key: String, value: ValueType) -> KeyValueItem {
        KeyValueItem {
            key,
            value,
            last_access_time: 1622657604, //TODO Esto debería calcularse
        }
    } //TODO chequear si es necesario este constructor
}

#[cfg(test)]
mod tests {
    use crate::key_value_item::key_value_item_domain::{KeyValueItem, ValueType};
    use std::collections::{HashSet, LinkedList};

    #[test]
    fn key_value_item_string_created() {
        let kv_item = KeyValueItem {
            key: "123".to_string(),
            value: ValueType::StringType("un_string".to_string()),
            last_access_time: 0,
        };

        assert_eq!(kv_item.value.to_string(), "un_string");
        assert_eq!(kv_item.key.to_string(), "123".to_string());
        assert_eq!(kv_item.last_access_time, 0);
    }

    #[test]
    fn key_value_item_set_created() {
        let mut un_set = HashSet::new();
        un_set.insert("un_set_string".to_string());

        let kv_item = KeyValueItem {
            key: "123".to_string(),
            value: ValueType::SetType(un_set),
            last_access_time: 0,
        };

        assert_eq!(kv_item.value.to_string(), "un_set_string");
        assert_eq!(kv_item.key.to_string(), "123".to_string());
        assert_eq!(kv_item.last_access_time, 0);
    }

    #[test]
    fn key_value_item_list_created() {
        let mut un_list = LinkedList::new();
        un_list.push_back("un_list_string".to_string());
        un_list.push_back("otro_list_string".to_string());

        let kv_item = KeyValueItem {
            key: "123".to_string(),
            value: ValueType::ListType(un_list),
            last_access_time: 0,
        };

        assert_eq!(kv_item.value.to_string(), "un_list_string,otro_list_string");
        assert_eq!(kv_item.key.to_string(), "123".to_string());
        assert_eq!(kv_item.last_access_time, 0);
    }
}
