use crate::domain::entities::key_value_item::{KeyAccessTime, KeyValueItem, ValueType};
use std::collections::{HashSet, LinkedList};

// Format: key; access_time; type; value
pub struct KeyValueItemSerialized {
    line: String,
}

impl KeyValueItemSerialized {
    pub fn _new(line: String) -> KeyValueItemSerialized {
        KeyValueItemSerialized { line }
    }
    pub fn transform_to_item(&self) -> KeyValueItem {
        // Format: key; access_time; type; value
        let line: Vec<&str> = self.line.split(';').collect();
        let value = match line[2] {
            "string" => ValueType::StringType(line[3].to_string()),
            "set" => {
                let mut hash_set = HashSet::new();
                let values: Vec<&str> = line[3].split(',').collect();
                for value in values {
                    hash_set.insert(value.to_string());
                }
                ValueType::SetType(hash_set)
            }
            "list" => {
                let mut list = LinkedList::new();
                let values: Vec<&str> = line[3].split(',').collect();
                for value in values {
                    list.push_back(value.to_string());
                }
                ValueType::ListType(list)
            }
            _ => panic!("Archivo corrupto. No pertenece a ningún tipo de dato soportado."),
        };

        KeyValueItem {
            key: line[0].to_string(),
            value,
            timeout: line[1].parse::<KeyAccessTime>().unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entities::key_value_item::{KeyAccessTime, ValueType};
    use crate::domain::entities::key_value_item_serialized::KeyValueItemSerialized;

    #[test]
    #[should_panic]
    fn line_has_no_valid_type() {
        let kvis = KeyValueItemSerialized::_new("123key;1623427130;no_type;value".to_string());
        kvis.transform_to_item();
    }

    #[test]
    fn line_string_type() {
        let kvis = KeyValueItemSerialized::_new("123key;1623427130;string;value".to_string());
        let kvi = kvis.transform_to_item();
        assert_eq!(kvi.get_key().to_string(), "123key");
        assert_eq!(kvi._get_value().to_string(), "value");
        assert_eq!(kvi._get_key_timeout().to_string(), "1623427130");
    }

    #[test]
    fn line_set_type() {
        let kvis = KeyValueItemSerialized::_new("123key;1623427130;set;3,2,4".to_string());
        let kvi = kvis.transform_to_item();
        assert_eq!(kvi.get_key().to_string(), "123key");
        match kvi.value {
            ValueType::SetType(hs) => {
                assert_eq!(hs.len(), 3);
                assert!(hs.contains("2"));
                assert!(hs.contains("3"));
                assert!(hs.contains("4"));
            }
            _ => assert!(false),
        }
        assert_eq!(kvi.timeout.to_string(), "1623427130");
    }

    #[test]
    fn line_list_type() {
        let kvis = KeyValueItemSerialized::_new("123key;1623427130;list;1,2,3".to_string());
        let kvi = kvis.transform_to_item();
        assert_eq!(kvi.key.to_string(), "123key");
        match kvi.value {
            ValueType::ListType(l) => {
                assert_eq!(l.len(), 3);
                let mut iter = l.iter();
                assert_eq!(iter.next(), Some(&"1".to_string()));
                assert_eq!(iter.next(), Some(&"2".to_string()));
                assert_eq!(iter.next(), Some(&"3".to_string()));
            }
            _ => assert!(false),
        }
        assert_eq!(kvi.timeout.to_string(), "1623427130");
    }

    #[test]
    fn line_persistent() {
        let kvis = KeyValueItemSerialized::_new("123key;;string;value".to_string());
        let kvi = kvis.transform_to_item();
        assert_eq!(kvi.key.to_string(), "123key");
        assert_eq!(kvi.value.to_string(), "value");
        match kvi.timeout {
            KeyAccessTime::Persistent => assert!(true),
            _ => assert!(false),
        }
    }
}
