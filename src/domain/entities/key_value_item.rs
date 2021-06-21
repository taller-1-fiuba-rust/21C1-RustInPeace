use crate::domain::entities::key_value_item_serialized::KeyValueItemSerialized;
use std::collections::{HashSet, LinkedList};
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;
use crate::errors::parse_error::ParseError::InvalidRequest;
use crate::errors::parse_error::ParseError;

#[allow(dead_code)]
#[derive(Debug, Clone)]
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
pub enum KeyAccessTime {
    Volatile(u64),
    Persistent,
}
impl fmt::Display for KeyAccessTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match self {
            KeyAccessTime::Volatile(value) => value.to_string(),
            KeyAccessTime::Persistent {} => "".to_string(),
        };
        write!(f, "{}", printable)
    }
}
impl FromStr for KeyAccessTime {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kat = match s {
            "" => KeyAccessTime::Persistent,
            _ => KeyAccessTime::Volatile(s.parse::<u64>().unwrap()),
        };
        Ok(kat)
    }
}

#[derive(Debug)]
pub struct KeyValueItem {
    pub(crate) key: String, //TODO tuve que hacer publicos estos atributos porque los necesito para testear
    pub(crate) value: ValueType,
    pub(crate) timeout: KeyAccessTime,
}

#[allow(dead_code)]
impl KeyValueItem {
    pub fn new(key: String, value: ValueType) -> KeyValueItem {
        KeyValueItem {
            key,
            value,
            timeout: KeyAccessTime::Volatile(1622657604), //TODO Esto debería calcularse
        }
    }
    pub fn _from_file(kvis: KeyValueItemSerialized) -> KeyValueItem {
        kvis.transform_to_item()
    }

    pub fn get_key(&self) -> &String {
        &self.key
    }
    pub fn set_timeout(&mut self, timeout: KeyAccessTime) -> bool{
        match timeout {
            KeyAccessTime::Persistent => false,
            KeyAccessTime::Volatile(_) => {
                self.timeout = timeout;
                true
            }
        }
    }

    pub fn _get_key_timeout(&self) -> &KeyAccessTime {
        &self.timeout
    }

    pub fn make_persistent(&mut self) -> bool {
        match self.timeout {
            KeyAccessTime::Persistent => false,
            KeyAccessTime::Volatile(_timeout) => {
                self.timeout = KeyAccessTime::Persistent;
                true
            }
        }
    }

    pub fn _get_value(&self) -> &ValueType {
        &self.value
    }

    pub fn get_copy_of_value(&self) -> ValueType {
        self.value.clone()
    }

    pub fn set_value(&mut self, new_value: ValueType) {
        self.value = new_value;
    }
}
#[cfg(test)]
mod tests {
    use crate::domain::entities::key_value_item::{KeyAccessTime, KeyValueItem, ValueType};
    use std::collections::{HashSet, LinkedList};

    #[test]
    fn key_value_item_string_created() {
        let kv_item = KeyValueItem {
            key: "123".to_string(),
            value: ValueType::StringType("un_string".to_string()),
            timeout: KeyAccessTime::Volatile(0),
        };

        assert_eq!(kv_item.value.to_string(), "un_string");
        assert_eq!(kv_item.key.to_string(), "123".to_string());
        match kv_item.timeout {
            KeyAccessTime::Persistent => assert!(false),
            KeyAccessTime::Volatile(timeout) => assert_eq!(timeout, 0),
        }
        assert_eq!(kv_item.timeout.to_string(), "0".to_string());
    }

    #[test]
    fn key_value_item_set_created() {
        let mut un_set = HashSet::new();
        un_set.insert("un_set_string".to_string());

        let kv_item = KeyValueItem {
            key: "123".to_string(),
            value: ValueType::SetType(un_set),
            timeout: KeyAccessTime::Volatile(0),
        };

        assert_eq!(kv_item.value.to_string(), "un_set_string");
        assert_eq!(kv_item.key.to_string(), "123".to_string());
        match kv_item.timeout {
            KeyAccessTime::Persistent => assert!(false),
            KeyAccessTime::Volatile(timeout) => assert_eq!(timeout, 0),
        }
        assert_eq!(kv_item.timeout.to_string(), "0".to_string());
    }

    #[test]
    fn key_value_item_list_created() {
        let mut un_list = LinkedList::new();
        un_list.push_back("un_list_string".to_string());
        un_list.push_back("otro_list_string".to_string());

        let kv_item = KeyValueItem {
            key: "123".to_string(),
            value: ValueType::ListType(un_list),
            timeout: KeyAccessTime::Volatile(0),
        };

        assert_eq!(kv_item.value.to_string(), "un_list_string,otro_list_string");
        assert_eq!(kv_item.key.to_string(), "123".to_string());
        match kv_item.timeout {
            KeyAccessTime::Persistent => assert!(false),
            KeyAccessTime::Volatile(timeout) => assert_eq!(timeout, 0),
        }
        assert_eq!(kv_item.timeout.to_string(), "0".to_string());
    }

    #[test]
    fn key_value_item_changes_to_persist() {
        let mut kv_item = KeyValueItem {
            key: "123".to_string(),
            value: ValueType::StringType("un_string".to_string()),
            timeout: KeyAccessTime::Volatile(0),
        };

        let res = kv_item.make_persistent();
        assert_eq!(res, true);
        match kv_item.timeout {
            KeyAccessTime::Volatile(_t) => assert!(false),
            KeyAccessTime::Persistent => assert!(true),
        }
        assert_eq!(kv_item.timeout.to_string(), "".to_string());
    }
}
