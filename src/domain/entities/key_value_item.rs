use std::collections::HashSet;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr; //, usize};
              //use crate::services::utils::resp_type::RespType;
// use crate::domain::entities::key_value_item_serialized::KeyValueItemSerialized;
// use std::collections::{HashSet, LinkedList};
// use std::fmt;
// use std::num::ParseIntError;
// use std::str::FromStr;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ValueType {
    ListType(Vec<String>),
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
pub struct ValueTimeItem {
    pub(crate) value: ValueType,
    pub(crate) last_access_time: KeyAccessTime,
}

impl ValueTimeItem {
    pub fn new(value: ValueType, time: KeyAccessTime) -> ValueTimeItem {
        ValueTimeItem {
// #[allow(dead_code)]
// impl KeyValueItem {
//     pub fn new(key: String, value: ValueType) -> KeyValueItem {
//         KeyValueItem {
//             key,
            value,
            last_access_time: time,//KeyAccessTime::Volatile(1622657604), //TODO Esto debería calcularse
        }
    }

    // pub fn _from_file(kvis: KeyValueItemSerialized) -> KeyValueItem {
    //     kvis.transform_to_item()
    // }

    // pub fn get_key(&self) -> &String {
    //     &self.key
    // }

    pub fn _get_last_access_time(&self) -> &KeyAccessTime {
        &self.last_access_time
    }

    pub fn make_persistent(&mut self) -> bool {
        match self.last_access_time {
            KeyAccessTime::Persistent => false,
            KeyAccessTime::Volatile(_timeout) => {
                self.last_access_time = KeyAccessTime::Persistent;
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

    pub fn _set_value(&mut self, new_value: ValueType) {
        self.value = new_value;
    }

    pub fn sort_descending(&self) -> Option<Vec<&String>> {
        let current_value = &self.value;
        match current_value {
            ValueType::ListType(current_list) => {
                //let mut vec = Vec::new();
                let mut vec: Vec<_> = current_list.into_iter().collect();
                vec.sort();
                vec.reverse();
                return Some(vec);
            }
            ValueType::SetType(current_set) => {
                let mut vec: Vec<_> = current_set.into_iter().collect();
                vec.sort();
                vec.reverse();
                return Some(vec);
            }
            _ => {
                return None;
            }
        }
    }

    pub fn sort(&self) -> Option<Vec<&String>> {
        let current_value_item = &self.value;
        match current_value_item {
            ValueType::ListType(current_list) => {
                let mut vec: Vec<_> = current_list.into_iter().collect();
                vec.sort();
                return Some(vec);
            }
            ValueType::SetType(current_set) => {
                let mut vec: Vec<_> = current_set.into_iter().collect();
                vec.sort();
                return Some(vec);
            }
            _ => {
                return None;
            }
        }
    }

    pub fn get_value_version_2(&self) -> Option<Vec<&String>> {
        let current_value_item = &self.value;
        match current_value_item {
            ValueType::ListType(current_list) => {
                let vec: Vec<_> = current_list.into_iter().collect();
                return Some(vec);
            }
            ValueType::SetType(current_set) => {
                let vec: Vec<_> = current_set.into_iter().collect();
                return Some(vec);
            }
            ValueType::StringType(current_string) => {
                let vec = vec![current_string];
                return Some(vec);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::domain::entities::key_value_item::{KeyAccessTime, ValueTimeItem, ValueType};
    use std::collections::HashSet;

    #[test]
    fn test_001_key_value_item_string_created() {
        let kv_item = ValueTimeItem {
            value: ValueType::StringType("un_string".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };

        assert_eq!(kv_item.value.to_string(), "un_string");
        match kv_item.last_access_time {
            KeyAccessTime::Persistent => assert!(false),
            KeyAccessTime::Volatile(timeout) => assert_eq!(timeout, 0),
        }
        assert_eq!(kv_item.last_access_time.to_string(), "0".to_string());
    }

    #[test]
    fn test_002_key_value_item_set_created() {
        let mut un_set = HashSet::new();
        un_set.insert("un_set_string".to_string());

        let kv_item = ValueTimeItem {
            value: ValueType::SetType(un_set),
            last_access_time: KeyAccessTime::Volatile(0),
        };

        assert_eq!(kv_item.value.to_string(), "un_set_string");
        match kv_item.last_access_time {
            KeyAccessTime::Persistent => assert!(false),
            KeyAccessTime::Volatile(timeout) => assert_eq!(timeout, 0),
        }
        assert_eq!(kv_item.last_access_time.to_string(), "0".to_string());
    }

    #[test]
    fn test_003_key_value_item_list_created() {
        let mut un_list = Vec::new();
        un_list.push("un_list_string".to_string());
        un_list.push("otro_list_string".to_string());

        let kv_item = ValueTimeItem {
            value: ValueType::ListType(un_list),
            last_access_time: KeyAccessTime::Volatile(0),
        };

        assert_eq!(kv_item.value.to_string(), "un_list_string,otro_list_string");
        match kv_item.last_access_time {
            KeyAccessTime::Persistent => assert!(false),
            KeyAccessTime::Volatile(timeout) => assert_eq!(timeout, 0),
        }
        assert_eq!(kv_item.last_access_time.to_string(), "0".to_string());
    }

    #[test]
    fn test_004_key_value_item_changes_to_persist() {
        let mut kv_item = ValueTimeItem {
            value: ValueType::StringType("un_string".to_string()),
            last_access_time: KeyAccessTime::Volatile(0),
        };

        let res = kv_item.make_persistent();
        assert_eq!(res, true);
        match kv_item.last_access_time {
            KeyAccessTime::Volatile(_t) => assert!(false),
            KeyAccessTime::Persistent => assert!(true),
        }
        assert_eq!(kv_item.last_access_time.to_string(), "".to_string());
    }

    #[test]
    fn test_005_list_of_numbers_is_sorted_ascending() {
        let kv_item = ValueTimeItem {
            value: ValueType::ListType(vec![
                20.to_string(),
                65.to_string(),
                1.to_string(),
                34.to_string(),
            ]),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let lista_ordenada = kv_item.sort().unwrap();
        println!("{:?}", lista_ordenada)
    }

    #[test]
    fn test_006_list_of_numbers_is_sorted_descending() {
        let kv_item = ValueTimeItem {
            value: ValueType::ListType(vec![
                20.to_string(),
                65.to_string(),
                1.to_string(),
                34.to_string(),
            ]),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let lista_ordenada_inversamente = kv_item.sort_descending().unwrap();
        println!("{:?}", lista_ordenada_inversamente)
    }

    #[test]
    fn test_007_list_of_words_is_sorted_inverse_abc() {
        let kv_item = ValueTimeItem {
            value: ValueType::ListType(vec![
                "juan".to_string(),
                "domingo".to_string(),
                "irma".to_string(),
                "dominga".to_string(),
            ]),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let lista_ordenada_inversamente = kv_item.sort_descending().unwrap();
        println!("{:?}", lista_ordenada_inversamente)
    }

    #[test]
    fn test_008_list_of_words_is_sorted_abc() {
        let kv_item = ValueTimeItem {
            value: ValueType::ListType(vec![
                "juan".to_string(),
                "domingo".to_string(),
                "irma".to_string(),
                "dominga".to_string(),
            ]),
            last_access_time: KeyAccessTime::Volatile(0),
        };
        let lista_ordenada = kv_item.sort().unwrap();
        println!("{:?}", lista_ordenada)
    }
}
