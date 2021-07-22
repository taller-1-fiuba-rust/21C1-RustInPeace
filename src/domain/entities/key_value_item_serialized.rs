//! Realiza la deserealización de los datos del arhivo dump

use crate::domain::entities::key_value_item::{
    KeyAccessTime, ValueTimeItem, ValueTimeItemBuilder, ValueType,
};
use std::collections::HashSet;
use std::str::FromStr;
/// Struct que representa una línea en el dump de la base de datos
pub struct KeyValueItemSerialized {
    line: String,
}
impl KeyValueItemSerialized {
    /// Constructor que admite como parámetro una línea en el archivo dump de la base de datos
    ///
    /// El formato necesario para que la linea represente una key value de redis es:
    ///
    /// "\<key\>;<last_access_time>;\<timeout\>;\<type\>;\<value\>"
    ///
    /// donde
    ///
    /// \<key\>: string único de identificación
    ///
    /// <last_access_time>: tiempo de último acceso a la key, con formato en timestamp
    ///
    /// \<timeout\>: tiempo de expiración de la clave, en el formato timestamp. Si la clave no expira el campo queda
    /// vacío
    ///
    /// \<type\>: tipo de valor almacenado en la key. Valores posibles: set, list o string.
    ///
    /// \<value\>: valor o valores almacenados en la key. Separados por coma.
    ///
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::entities::key_value_item_serialized::KeyValueItemSerialized;
    ///
    ///
    /// let kvis = KeyValueItemSerialized::new("123key;1623427130;1623427130;set;3,2,4".to_string());
    /// ```
    pub fn new(line: String) -> KeyValueItemSerialized {
        KeyValueItemSerialized { line }
    }

    /// Método que tranforma un KeyValueItemSerialized en una tupla (key,value)
    ///
    /// A partir de la línea obtenida en el dump de la base de datos, se invoca a este método para
    /// hacer la deserealización correpondiente.
    ///
    /// Si el tipo de dato leido no es uno de los 3 posibles (set, string o list) la función
    /// retornará un panic.
    ///
    /// # Example
    ///
    /// ```
    /// use proyecto_taller_1::domain::entities::key_value_item_serialized::KeyValueItemSerialized;
    /// use proyecto_taller_1::domain::entities::key_value_item::ValueType::SetType;
    /// use proyecto_taller_1::domain::entities::key_value_item::ValueType;
    ///
    ///
    /// let kvis = KeyValueItemSerialized::new("123key;1623427130;1623427130;set;3,2,4".to_string());
    /// let kvi = kvis.transform_to_item();
    ///
    ///  assert_eq!(kvi.0.to_string(), "123key");
    ///  if let SetType(_) = kvi.1.get_value(){assert!(true)}else{ assert!(false)}
    ///  assert_eq!(kvi.1.get_timeout().to_string(), "1623427130");
    /// ```
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
                    ValueTimeItemBuilder::new(value)
                        .with_key_access_time(timeout)
                        .with_last_access_time(last_access_time)
                        .build(),
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
