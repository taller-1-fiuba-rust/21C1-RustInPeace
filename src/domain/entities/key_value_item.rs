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
/// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, KeyAccessTime, ValueType, ValueTimeItemBuilder};
///
///
/// let mut un_list = Vec::new();
///  un_list.push("primer_elemento".to_string());
///  un_list.push("segundo_elemento".to_string());
///
///  let kv_item = ValueTimeItemBuilder::new(ValueType::ListType(un_list)).build();
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
/// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, KeyAccessTime, ValueType, ValueTimeItemBuilder};
///
///
/// let mut un_list = Vec::new();
///  un_list.push("primer_elemento".to_string());
///  un_list.push("segundo_elemento".to_string());
///
///  let kv_item = ValueTimeItemBuilder::new(ValueType::ListType(un_list)).with_timeout(123210).build();
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
/// Formato de deserealización para los tipos de key almacenados.
///
/// Si la clave es de tipo `volátil` se lee el valor de tiempo de expiración
/// y se parsea en u64.
/// En el caso de las de tipo `persistente` se leerá un string vacío
/// indicando que no hay tiempo de expiración para ella.
///
/// ```
/// use proyecto_taller_1::domain::entities::key_value_item::KeyAccessTime;
/// use std::str::FromStr;
///
/// let line = "1211111";
/// match line.parse::<KeyAccessTime>().unwrap(){
///     KeyAccessTime::Volatile(_) => assert!(true),
///     _ => assert!(false)
/// };
/// ```
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

/// Representa el objeto guardado en una key.
///
/// Contiene 3 atributos:
///
/// value: Tipo de dato ValueType
/// timeout: Tipo de dato KeyAccessTime
/// last_access_time: Tipo de dato u64. Es el timestamp del último acceso a la key
#[derive(Debug)]
pub struct ValueTimeItem {
    value: ValueType,
    timeout: KeyAccessTime,
    last_access_time: u64,
}
/// Builder para ValueTimeItem
///
/// Permite construir el valueTimeItem por partes
/// Contiene 3 elementos:
/// value: Tipo de dato ValueType
/// timeout: Tipo de dato KeyAccessTime
/// last_access_time: Tipo de dato u64. Es el timestamp del último acceso a la key
pub struct ValueTimeItemBuilder {
    value: ValueType,
    timeout: KeyAccessTime,
    last_access_time: u64,
}

impl ValueTimeItemBuilder {
    /// Constructor defaulta para ValueTimeItemBuilder
    /// Este constructor guarda automáticamente el tiempo de último acceso de
    /// de la clave con now y considera que la clave es de tipo `Persistent`
    ///
    /// # Ejemplo
    /// ```
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let kv_item = ValueTimeItemBuilder::new(ValueType::ListType(vec!["elemento".to_string()])).build();
    /// ```
    pub fn new(value: ValueType) -> ValueTimeItemBuilder {
        ValueTimeItemBuilder {
            value,
            timeout: KeyAccessTime::Persistent,
            last_access_time: {
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            },
        }
    }
    /// Permite agregar un timeout a la clave.
    ///
    /// Crea la clave en tipo volátil y le setea el timeout indicado.
    ///
    /// # Ejemplo
    /// ```
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// ValueTimeItemBuilder::new(ValueType::ListType(vec!["element1".to_string()])).with_timeout(1623433677).build();
    /// ```
    pub fn with_timeout(mut self, timeout: u64) -> ValueTimeItemBuilder {
        self.timeout = KeyAccessTime::Volatile(timeout);
        self
    }

    /// Permite agregar un tiempo de último acceso a la clave.
    ///
    /// Le agrega al builder un tiempo de último acceso particular.
    ///
    /// # Ejemplo
    /// ```
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// ValueTimeItemBuilder::new(ValueType::ListType(vec!["element".to_string()])).with_last_access_time(1623433677).build();
    /// ```
    pub fn with_last_access_time(mut self, lat: u64) -> ValueTimeItemBuilder {
        self.last_access_time = lat;
        self
    }
    /// Permite agregar un tipo de clave específico: Persistente o Volátil.
    ///
    /// Le agrega al builder un tipo de clave en particular.
    ///
    /// # Ejemplo
    /// ```
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let kv_item = ValueTimeItemBuilder::new(ValueType::ListType(vec!["elemento".to_string()]))
    ///     .with_key_access_time(KeyAccessTime::Volatile(1623433677))
    ///     .build();
    /// ```
    pub fn with_key_access_time(mut self, kat: KeyAccessTime) -> ValueTimeItemBuilder {
        self.timeout = kat;
        self
    }
    /// Transforma un ValueTimeItemBuilder en ValueTimeItem
    ///
    /// A partir de las configuraciones del builder, crea una instancia con
    /// toda la data.
    pub fn build(self) -> ValueTimeItem {
        ValueTimeItem {
            timeout: self.timeout,
            value: self.value,
            last_access_time: self.last_access_time,
        }
    }
}

/// Representa un valor que puede ser almacenado.
/// Se compone por un tipo de valor que puede ser String, Set o List, por un timeout y un last access time.
impl ValueTimeItem {
    pub fn _from_file(kvis: KeyValueItemSerialized) -> (String, ValueTimeItem) {
        kvis.transform_to_item()
    }

    /// Establece un tiempo de timeout.
    /// Si el tiempo es de tipo Persistente, entonces el valor no expira nunca.
    /// Si el tiempo es de tipo Volatile, el valor expirará después de `timeout` tiempo.
    pub fn set_timeout(&mut self, timeout: KeyAccessTime) -> bool {
        match timeout {
            KeyAccessTime::Persistent => false,
            KeyAccessTime::Volatile(_) => {
                self.timeout = timeout;
                true
            }
        }
    }

    /// Devuelve el timeout asociado.
    pub fn get_timeout(&self) -> &KeyAccessTime {
        &self.timeout
    }

    /// Devuelve el tiempo en que se realizó el último acceso al valor.
    pub fn get_last_access_time(&self) -> &u64 {
        &self.last_access_time
    }

    /// Reinicia el valor del último tiempo de acceso.
    pub fn reboot_last_access_time(&mut self) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_access_time = now;
    }

    /// Transforma el valor a tipo Persistente.
    ///
    /// Si el valor ya era persistente, devuelve False.
    /// Si el valor tenía un timeout asociado, lo elimina y lo hace persistente.
    pub fn make_persistent(&mut self) -> bool {
        match self.timeout {
            KeyAccessTime::Persistent => false,
            KeyAccessTime::Volatile(_timeout) => {
                self.timeout = KeyAccessTime::Persistent;
                true
            }
        }
    }

    /// Devuelve el valor almacenado.
    pub fn get_value(&self) -> &ValueType {
        &self.value
    }

    /// Devuelve una copia del valor almacenado.
    pub fn get_copy_of_value(&self) -> ValueType {
        self.value.clone()
    }

    /// Devuelve el tiempo de acceso asociado, ya sea volatil o no.
    pub fn get_copy_of_timeout(&self) -> KeyAccessTime {
        match self.timeout {
            KeyAccessTime::Persistent => KeyAccessTime::Persistent,
            KeyAccessTime::Volatile(timeout) => KeyAccessTime::Volatile(timeout),
        }
    }

    /// Devuelve si expiró.
    ///
    /// Compara el tiempo de timeout con el tiempo actual para verificar si expiró.
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

    /// Actualiza el valor.
    pub fn set_value(&mut self, new_value: ValueType) {
        self.value = new_value;
    }

    /// Devuelve una lista ordenada en forma descendente del valor.
    ///
    /// Si el valor es de tipo string, devuelve el valor original.
    /// Si es de tipo Set o List, lo devuelve ordenado en forma descendente.
    pub fn sort_descending(&self) -> Vec<String> {
        let current_value = self.value.clone();
        match current_value {
            ValueType::ListType(mut current_list) => {
                current_list.sort();
                current_list.reverse();
                current_list
            }
            ValueType::SetType(current_set) => {
                let mut vec: Vec<String> = current_set.into_iter().collect();
                vec.sort();
                vec.reverse();
                vec
            }
            ValueType::StringType(current_string) => vec![current_string],
        }
    }

    /// Devuelve una lista ordenada en forma ascendente del valor.
    ///
    /// Si el valor es de tipo string, devuelve el valor original.
    /// Si es de tipo Set o List, lo devuelve ordenado en forma ascendente.
    pub fn sort(&self) -> Vec<String> {
        let current_value_item = self.value.clone();
        match current_value_item {
            ValueType::ListType(mut current_list) => {
                current_list.sort();
                current_list
            }
            ValueType::SetType(current_set) => {
                let mut vec: Vec<String> = current_set.into_iter().collect();
                vec.sort();
                vec
            }
            ValueType::StringType(current_string) => vec![current_string],
        }
    }

    /// Devuelve el valor en forma de vector.
    pub fn get_value_as_vec(&self) -> Vec<&String> {
        let current_value_item = &self.value;
        match current_value_item {
            ValueType::ListType(current_list) => current_list.iter().collect(),
            ValueType::SetType(current_set) => current_set.iter().collect(),
            ValueType::StringType(current_string) => vec![current_string],
        }
    }

    /// Devuelve el tipo de valor en forma de string.
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

#[test]
fn test_001_key_value_item_string_created() {
    use crate::domain::entities::key_value_item::{KeyAccessTime, ValueTimeItemBuilder, ValueType};

    let kv_item = ValueTimeItemBuilder::new(ValueType::StringType("un_string".to_string()))
        .with_timeout(0)
        .build();

    assert_eq!(kv_item.get_value().to_string(), "un_string");

    match kv_item.get_timeout() {
        KeyAccessTime::Persistent => assert!(false),
        KeyAccessTime::Volatile(timeout) => assert_eq!(timeout, &0_u64),
    }
    assert_eq!(kv_item.get_timeout().to_string(), "0".to_string());
}

#[test]
fn test_002_key_value_item_set_created() {
    use crate::domain::entities::key_value_item::{KeyAccessTime, ValueTimeItemBuilder, ValueType};
    use std::collections::HashSet;

    let mut un_set = HashSet::new();
    un_set.insert("un_set_string".to_string());

    let kv_item = ValueTimeItemBuilder::new(ValueType::SetType(un_set))
        .with_timeout(0)
        .build();
    assert_eq!(kv_item.value.to_string(), "un_set_string");

    match kv_item.timeout {
        KeyAccessTime::Persistent => assert!(false),
        KeyAccessTime::Volatile(timeout) => assert_eq!(timeout, 0),
    }
    assert_eq!(kv_item.timeout.to_string(), "0".to_string());
}

#[test]
fn test_003_key_value_item_list_created() {
    use crate::domain::entities::key_value_item::{KeyAccessTime, ValueTimeItemBuilder, ValueType};
    let mut un_list = Vec::new();
    un_list.push("un_list_string".to_string());
    un_list.push("otro_list_string".to_string());

    let kv_item = ValueTimeItemBuilder::new(ValueType::ListType(un_list))
        .with_timeout(0)
        .build();

    assert_eq!(kv_item.value.to_string(), "un_list_string,otro_list_string");
    match kv_item.timeout {
        KeyAccessTime::Persistent => assert!(false),
        KeyAccessTime::Volatile(timeout) => assert_eq!(timeout, 0),
    }
    assert_eq!(kv_item.timeout.to_string(), "0".to_string());
}

#[test]
fn test_004_key_value_item_changes_to_persist() {
    use crate::domain::entities::key_value_item::{KeyAccessTime, ValueTimeItemBuilder, ValueType};

    let mut kv_item = ValueTimeItemBuilder::new(ValueType::StringType("un_string".to_string()))
        .with_timeout(0)
        .build();

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
    use crate::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType};

    let kv_item = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        20.to_string(),
        65.to_string(),
        1.to_string(),
        34.to_string(),
    ]))
    .with_timeout(0)
    .build();

    let lista_ordenada = kv_item.sort();
    assert_eq!(
        lista_ordenada,
        vec![
            String::from("1"),
            String::from("20"),
            String::from("34"),
            String::from("65")
        ]
    );
}

#[test]
fn test_006_list_of_numbers_is_sorted_descending() {
    use crate::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType};

    let kv_item = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        20.to_string(),
        65.to_string(),
        1.to_string(),
        34.to_string(),
    ]))
    .with_timeout(0)
    .build();

    let lista_ordenada_inversamente = kv_item.sort_descending();
    assert_eq!(
        lista_ordenada_inversamente,
        vec![
            String::from("65"),
            String::from("34"),
            String::from("20"),
            String::from("1")
        ]
    );
}

#[test]
fn test_007_list_of_words_is_sorted_inverse_abc() {
    use crate::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType};

    let kv_item = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "juan".to_string(),
        "domingo".to_string(),
        "irma".to_string(),
        "dominga".to_string(),
    ]))
    .with_timeout(0)
    .build();

    let lista_ordenada_inversamente = kv_item.sort_descending();
    assert_eq!(
        lista_ordenada_inversamente,
        vec![
            String::from("juan"),
            String::from("irma"),
            String::from("domingo"),
            String::from("dominga")
        ]
    )
}

#[test]
fn test_008_list_of_words_is_sorted_abc() {
    use crate::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType};

    let kv_item = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "juan".to_string(),
        "domingo".to_string(),
        "irma".to_string(),
        "dominga".to_string(),
    ]))
    .with_timeout(0)
    .build();
    let lista_ordenada = kv_item.sort();
    assert_eq!(
        lista_ordenada,
        vec![
            String::from("dominga"),
            String::from("domingo"),
            String::from("irma"),
            String::from("juan")
        ]
    );
}
