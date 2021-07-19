//! Representa los valores almacenados en la base de datos

use crate::domain::entities::key_value_item_serialized::KeyValueItemSerialized;
use std::collections::HashSet;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::SystemTime;

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Tipos de value almacenados
///
/// Los posibles valores son: List, Set, String.
/// Dentro de las listas o los sets, los valores son de tipo String.
pub enum ValueType {
    ListType(Vec<String>),
    SetType(HashSet<String>),
    StringType(String),
}

/// Formato display para los valores almacenados.
///
/// La lista de valores se imprimen uno tras otro
/// separados por comas.
/// En el caso de string, solo se imprimirá un valor. Para Set no existe orden
/// y en el caso de las listas se imprime primero el elemento del head
/// hasta ir avanzando al final.
///
/// # Example
///
/// ```
/// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, KeyAccessTime, ValueType};
///
///
/// let mut un_list = Vec::new();
///  un_list.push("primer_elemento".to_string());
///  un_list.push("segundo_elemento".to_string());
///
///  let kv_item = ValueTimeItem::new_now(ValueType::ListType(un_list), KeyAccessTime::Volatile(0));
///  assert_eq!(kv_item.get_value().to_string(), "primer_elemento,segundo_elemento");
///
/// ```
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
/// Tipos de key almacenados
///
/// Los posibles valores son: Volátil (almacena el timeout de expiración)
/// Persistente: este tipo de claves no expiran.
pub enum KeyAccessTime {
    Volatile(u64),
    Persistent,
}

/// Formato display para los tipos de key almacenados.
///
/// Si la clave es de tipo `volátil` se imprime el tiempo de expiración.
/// En el caso de las de tipo `persistente` no se imprimirá ningún valor
/// indicando que no hay tiempo de expiración para ella.
///
/// ```
/// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, KeyAccessTime, ValueType};
///
///
/// let mut un_list = Vec::new();
///  un_list.push("primer_elemento".to_string());
///  un_list.push("segundo_elemento".to_string());
///
///  let kv_item = ValueTimeItem::new_now(ValueType::ListType(un_list), KeyAccessTime::Volatile(123210));
///  assert_eq!(kv_item.get_value().to_string(), "primer_elemento,segundo_elemento");
///  assert_eq!(kv_item.get_timeout().to_string(), "123210".to_string());
/// ```
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
/// Representa el objeto guardado en una key.
///
/// Contiene 3 atributos:
///
/// value: Tipo de dato ValueType
/// timeout: Tipo de dato KeyAccessTime
/// last_access_time: Tipo de dato u64. Es el timestamp del último acceso a la key
pub struct ValueTimeItem {
    value: ValueType,
    timeout: KeyAccessTime,
    last_access_time: u64,
}

impl ValueTimeItem {
    pub fn new_now(value: ValueType, time: KeyAccessTime) -> ValueTimeItem {
        ValueTimeItem {
            value,
            timeout: time,
            last_access_time: {
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            },
        }
    }
    pub fn new(value: ValueType, time: KeyAccessTime, last_access_time: u64) -> ValueTimeItem {
        ValueTimeItem {
            value,
            timeout: time,
            last_access_time,
        }
    }

    pub fn _from_file(kvis: KeyValueItemSerialized) -> (String, ValueTimeItem) {
        kvis.transform_to_item()
    }
    pub fn set_timeout(&mut self, timeout: KeyAccessTime) -> bool {
        match timeout {
            KeyAccessTime::Persistent => false,
            KeyAccessTime::Volatile(_) => {
                self.timeout = timeout;
                true
            }
        }
    }

    pub fn get_timeout(&self) -> &KeyAccessTime {
        &self.timeout
    }

    pub fn get_last_access_time(&self) -> &u64 {
        &self.last_access_time
    }

    pub fn reboot_last_access_time(&mut self) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_access_time = now;
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

    pub fn get_value(&self) -> &ValueType {
        &self.value
    }

    pub fn get_copy_of_value(&self) -> ValueType {
        self.value.clone()
    }
    pub fn get_copy_of_timeout(&self) -> KeyAccessTime {
        match self.timeout {
            KeyAccessTime::Persistent => KeyAccessTime::Persistent,
            KeyAccessTime::Volatile(timeout) => KeyAccessTime::Volatile(timeout),
        }
    }
    pub fn is_expired(&self) -> bool {
        let kat = self.get_timeout();
        if let KeyAccessTime::Volatile(timeout) = kat {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            return timeout < &now;
        }
        false
    }

    pub fn set_value(&mut self, new_value: ValueType) {
        self.value = new_value;
    }

    pub fn sort_descending(&self) -> Option<Vec<&String>> {
        let current_value = &self.value;
        match current_value {
            ValueType::ListType(current_list) => {
                //let mut vec = Vec::new();
                let mut vec: Vec<_> = current_list.iter().collect();
                vec.sort();
                vec.reverse();
                Some(vec)
            }
            ValueType::SetType(current_set) => {
                let mut vec: Vec<_> = current_set.iter().collect();
                vec.sort();
                vec.reverse();
                Some(vec)
            }
            _ => None,
        }
    }

    pub fn sort(&self) -> Option<Vec<&String>> {
        let current_value_item = &self.value;
        match current_value_item {
            ValueType::ListType(current_list) => {
                let mut vec: Vec<_> = current_list.iter().collect();
                vec.sort();
                Some(vec)
            }
            ValueType::SetType(current_set) => {
                let mut vec: Vec<_> = current_set.iter().collect();
                vec.sort();
                Some(vec)
            }
            _ => None,
        }
    }

    pub fn get_value_version_2(&self) -> Option<Vec<&String>> {
        let current_value_item = &self.value;
        match current_value_item {
            ValueType::ListType(current_list) => {
                let vec: Vec<_> = current_list.iter().collect();
                Some(vec)
            }
            ValueType::SetType(current_set) => {
                let vec: Vec<_> = current_set.iter().collect();
                Some(vec)
            }
            ValueType::StringType(current_string) => {
                let vec = vec![current_string];
                Some(vec)
            }
        }
    }

    pub fn get_value_type(&self) -> String {
        let value_type;
        let current_value = &self.value;
        match current_value {
            ValueType::ListType(_current_list) => {
                value_type = "list".to_string();
            }
            ValueType::SetType(_current_set) => {
                value_type = "set".to_string();
            }
            ValueType::StringType(_current_string) => {
                value_type = "string".to_string();
            }
        }
        value_type
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entities::key_value_item::{KeyAccessTime, ValueTimeItem, ValueType};
    use std::collections::HashSet;

    #[test]
    fn test_001_key_value_item_string_created() {
        let kv_item = ValueTimeItem::new_now(
            ValueType::StringType("un_string".to_string()),
            KeyAccessTime::Volatile(0),
        );

        assert_eq!(kv_item.value.to_string(), "un_string");

        match kv_item.timeout {
            KeyAccessTime::Persistent => assert!(false),
            KeyAccessTime::Volatile(timeout) => assert_eq!(timeout, 0),
        }
        assert_eq!(kv_item.timeout.to_string(), "0".to_string());
    }

    #[test]
    fn test_002_key_value_item_set_created() {
        let mut un_set = HashSet::new();
        un_set.insert("un_set_string".to_string());

        let kv_item =
            ValueTimeItem::new_now(ValueType::SetType(un_set), KeyAccessTime::Volatile(0));
        assert_eq!(kv_item.value.to_string(), "un_set_string");

        match kv_item.timeout {
            KeyAccessTime::Persistent => assert!(false),
            KeyAccessTime::Volatile(timeout) => assert_eq!(timeout, 0),
        }
        assert_eq!(kv_item.timeout.to_string(), "0".to_string());
    }

    #[test]
    fn test_003_key_value_item_list_created() {
        let mut un_list = Vec::new();
        un_list.push("un_list_string".to_string());
        un_list.push("otro_list_string".to_string());

        let kv_item =
            ValueTimeItem::new_now(ValueType::ListType(un_list), KeyAccessTime::Volatile(0));

        assert_eq!(kv_item.value.to_string(), "un_list_string,otro_list_string");

        match kv_item.timeout {
            KeyAccessTime::Persistent => assert!(false),
            KeyAccessTime::Volatile(timeout) => assert_eq!(timeout, 0),
        }
        assert_eq!(kv_item.timeout.to_string(), "0".to_string());
    }

    #[test]
    fn test_004_key_value_item_changes_to_persist() {
        let mut kv_item = ValueTimeItem::new_now(
            ValueType::StringType("un_string".to_string()),
            KeyAccessTime::Volatile(0),
        );

        let res = kv_item.make_persistent();
        assert_eq!(res, true);
        match kv_item.timeout {
            KeyAccessTime::Volatile(_t) => assert!(false),
            KeyAccessTime::Persistent => assert!(true),
        }
        assert_eq!(kv_item.timeout.to_string(), "".to_string());
    }

    #[test]
    fn test_005_list_of_numbers_is_sorted_ascending() {
        let kv_item = ValueTimeItem::new_now(
            ValueType::ListType(vec![
                20.to_string(),
                65.to_string(),
                1.to_string(),
                34.to_string(),
            ]),
            KeyAccessTime::Volatile(0),
        );

        let lista_ordenada = kv_item.sort().unwrap();
        println!("{:?}", lista_ordenada)
    }

    #[test]
    fn test_006_list_of_numbers_is_sorted_descending() {
        let kv_item = ValueTimeItem::new_now(
            ValueType::ListType(vec![
                20.to_string(),
                65.to_string(),
                1.to_string(),
                34.to_string(),
            ]),
            KeyAccessTime::Volatile(0),
        );
        let lista_ordenada_inversamente = kv_item.sort_descending().unwrap();
        println!("{:?}", lista_ordenada_inversamente)
    }

    #[test]
    fn test_007_list_of_words_is_sorted_inverse_abc() {
        let kv_item = ValueTimeItem::new_now(
            ValueType::ListType(vec![
                "juan".to_string(),
                "domingo".to_string(),
                "irma".to_string(),
                "dominga".to_string(),
            ]),
            KeyAccessTime::Volatile(0),
        );
        let lista_ordenada_inversamente = kv_item.sort_descending().unwrap();
        println!("{:?}", lista_ordenada_inversamente)
    }

    #[test]
    fn test_008_list_of_words_is_sorted_abc() {
        let kv_item = ValueTimeItem::new_now(
            ValueType::ListType(vec![
                "juan".to_string(),
                "domingo".to_string(),
                "irma".to_string(),
                "dominga".to_string(),
            ]),
            KeyAccessTime::Volatile(0),
        );
        let lista_ordenada = kv_item.sort().unwrap();
        println!("{:?}", lista_ordenada)
    }
}
