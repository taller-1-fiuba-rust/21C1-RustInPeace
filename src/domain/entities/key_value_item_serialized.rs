use crate::domain::entities::key_value_item::{KeyAccessTime, ValueTimeItem, ValueType};
use std::collections::HashSet;
use std::str::FromStr;

// Format: key; access_time; type; value
pub struct KeyValueItemSerialized {
    line: String,
}

impl KeyValueItemSerialized {
    pub fn new(line: String) -> KeyValueItemSerialized {
        KeyValueItemSerialized { line }
    }
    pub fn transform_to_item(&self) -> (String, ValueTimeItem) {
        // Format: key; last_access_time; timeout; type; value
        let line: Vec<&str> = self.line.split(';').collect();
        let value = match line[3] {
            "string" => ValueType::StringType(line[4].to_string()),
            "set" => {
                let mut hash_set = HashSet::new();
                let values: Vec<&str> = line[4].split(',').collect();
                for value in values {
                    hash_set.insert(value.to_string());
                }
                ValueType::SetType(hash_set)
            }
            "list" => {
                let mut list = Vec::new();
                let values: Vec<&str> = line[4].split(',').collect();
                for value in values {
                    list.push(value.to_string());
                }
                ValueType::ListType(list)
            }
            _ => panic!("Archivo corrupto. No pertenece a ningún tipo de dato soportado."),
        };
        let last_access_time_r = u64::from_str(line[1]);
        match last_access_time_r {
            Ok(last_access_time) => {
                let timeout = line[2].parse::<KeyAccessTime>().unwrap();
                (
                    line[0].to_string(),
                    ValueTimeItem::new(value, timeout, last_access_time),
                )
            }
            Err(_) => {
                panic!("Archivo corrupto. No se pudo levantar el last_access_time.")
            }
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
        let kvis =
            KeyValueItemSerialized::new("123key;1623427130;1623427130;no_type;value".to_string());
        kvis.transform_to_item();
    }

    #[test]
    fn line_string_type() {
        let kvis =
            KeyValueItemSerialized::new("123key;1623427130;1623427130;string;value".to_string());
        let kvi = kvis.transform_to_item();

        assert_eq!(kvi.0.to_string(), "123key");
        assert_eq!(kvi.1.get_value().to_string(), "value");
        assert_eq!(kvi.1.get_timeout().to_string(), "1623427130");
    }

    #[test]
    fn line_set_type() {
        let kvis =
            KeyValueItemSerialized::new("123key;1623427130;1623427130;set;3,2,4".to_string());
        let kvi = kvis.transform_to_item();

        assert_eq!(kvi.0.to_string(), "123key");
        match kvi.1.get_value() {
            ValueType::SetType(hs) => {
                assert_eq!(hs.len(), 3);
                assert!(hs.contains("2"));
                assert!(hs.contains("3"));
                assert!(hs.contains("4"));
            }
            _ => assert!(false),
        }
        assert_eq!(kvi.1.get_timeout().to_string(), "1623427130");
    }

    #[test]
    fn line_list_type() {
        let kvis =
            KeyValueItemSerialized::new("123key;1623427130;1623427130;list;1,2,3".to_string());
        let kvi = kvis.transform_to_item();
        assert_eq!(kvi.0.to_string(), "123key");
        match kvi.1.get_value() {
            ValueType::ListType(l) => {
                assert_eq!(l.len(), 3);
                let mut iter = l.iter();
                assert_eq!(iter.next(), Some(&"1".to_string()));
                assert_eq!(iter.next(), Some(&"2".to_string()));
                assert_eq!(iter.next(), Some(&"3".to_string()));
            }
            _ => assert!(false),
        }

        assert_eq!(kvi.1.get_timeout().to_string(), "1623427130");
    }

    #[test]
    fn line_persistent() {
        let kvis = KeyValueItemSerialized::new("123key;1623427130;;string;value".to_string());
        let kvi = kvis.transform_to_item();

        assert_eq!(kvi.0.to_string(), "123key");
        assert_eq!(kvi.1.get_value().to_string(), "value");
        match kvi.1.get_timeout() {
            KeyAccessTime::Persistent => assert!(true),
            _ => assert!(false),
        }
    }
}
