use crate::domain::entities::key_value_item::KeyAccessTime;
use crate::domain::entities::key_value_item::{ValueTimeItem, ValueType};
use crate::domain::entities::key_value_item_serialized::KeyValueItemSerialized;
use crate::errors::database_error::DatabaseError;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::Write;
use std::io::{self};
use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;

#[derive(Debug)]
pub struct Database {
    dbfilename: String,
    items: HashMap<String, ValueTimeItem>,
}

impl Database {
    pub fn new(filename: String) -> Database {
        let mut db = Database {
            dbfilename: filename,
            items: HashMap::new(),
        };
        db.load_items();
        db
    }

    pub fn _get_filename(&self) -> String {
        self.dbfilename.clone()
    }

    pub fn get_items(&self) -> &HashMap<String, ValueTimeItem> {
        &self.items
    }

    pub fn get_live_item(&mut self, key: &str) -> Option<&ValueTimeItem> {
        let items = self.check_timeout_item(key);
        if items.is_none() {
            let _ = self.items.remove(key);
        }
        self.items.get(key)
    }

    pub fn get_mut_live_item(&mut self, key: &str) -> Option<&mut ValueTimeItem> {
        let items = self.check_timeout_item(key);
        if items.is_none() {
            let _ = self.items.remove(key);
        }
        self.items.get_mut(key)
    }

    pub fn check_timeout_item(&mut self, key: &str) -> Option<&ValueTimeItem> {
        let option_item = self.items.get(key);
        match option_item {
            Some(item) => {
                if item.is_expired() {
                    None
                } else {
                    Some(item)
                }
            }
            None => None,
        }
    }

    /// borra todos las claves (y sus valores asociados) de la base de datos
    pub fn clean_items(&mut self) -> &HashMap<String, ValueTimeItem> {
        self.items.clear();
        &self.items
    }

    /// Devuelve las claves que hacen *match* con un *pattern* sin uso de regex (limitado)
    pub fn get_keys_that_match_pattern_sin_regex(&self, pattern: String) -> Vec<String> {
        let mut vector_keys = vec![];
        for key in &self.items {
            let current_key = key.to_owned().0.to_string();
            if !key.1.is_expired() {
                vector_keys.push(current_key);
            }
        }
        let mut vector_keys_filtered = vec![];
        for key in vector_keys {
            if key.contains(&pattern.to_string()) {
                vector_keys_filtered.push(key);
            }
        }
        vector_keys_filtered
    }

    /// Devuelve las claves que hacen *match* con un *pattern* con uso de regex (exhaustivo)
    pub fn get_keys_that_match_pattern(&self, pattern: &str) -> Vec<String> {
        let mut vec_matching_keys = vec![];
        //aca agarro todas las claves disponibles en un vector
        let mut vector_keys = vec![];
        for key in &self.items {
            let current_key = key.to_owned().0.to_string();
            if !key.1.is_expired() {
                vector_keys.push(current_key);
            }
        }
        //aca genero el regex a partir de pattern y lo comparo contra todas las claves
        let re = Regex::new(pattern).unwrap();
        for key in vector_keys {
            if re.is_match(&key) {
                vec_matching_keys.push(key);
            }
        }
        vec_matching_keys
    }

    ///devuelve **true** si la clave existe en *database*
    pub fn key_exists(&mut self, key: String) -> bool {
        return self.get_live_item(&key).is_some();
    }

    ///devuelve **true** si la clave existe en *database*
    // pub fn key_exists_2(&self, key: String) -> bool {
    //     return self.items.contains_key(&key); // (&key).is_some();
    // }
    /// permite agregar *clave* y *valor* a la base de datos
    pub fn add(&mut self, key: String, value: ValueTimeItem) {
        self.items.insert(key, value);
    }

    /// obtiene las claves de la **db** que hacen *match* con el **pat** + **element** (de
    /// **elements** y devuelve una tupla con (**element**,**patterned_key_value**)
    pub fn get_values_of_external_keys_that_match_a_pattern(
        &self,
        elements: Vec<String>,
        pat: &str,
    ) -> Option<Vec<(String, String)>> {
        let mut vec_auxiliar = Vec::new();
        for element in elements {
            let patterned_key = pat.to_string() + element.as_str();
            println!("patterned key: {:?}", &patterned_key);
            if self.items.contains_key(&patterned_key) {
                let current_value = self
                    .items
                    .get(&patterned_key)
                    .unwrap()
                    .get_value_version_2()
                    .unwrap();
                let vectorcito = (element, current_value[0].to_string());
                vec_auxiliar.push(vectorcito);
            }
        }
        if !vec_auxiliar.is_empty() {
            Some(vec_auxiliar)
        } else {
            None
        }
    }

    pub fn get_values_and_associated_external_key_values(
        &mut self,
        pat: String,
        key: String,
    ) -> Option<Vec<(String, String)>> {
        //aca agarro la key que me pasan por request
        let my_list_value_optional = self.get_live_item(&key);
        if let Some(my_list_value) = my_list_value_optional {
            let elements = my_list_value.get_value_version_2().unwrap(); //aca tengo la lista guardada en la key q me pasaron
            let mut vec_resptype_to_string = Vec::new();
            for aux in elements {
                vec_resptype_to_string.push(aux.to_string());
            }
            let tuple_vector = self
                .get_values_of_external_keys_that_match_a_pattern(vec_resptype_to_string, &pat)
                .unwrap();
            Some(tuple_vector)
        } else {
            None
        }
    }

    pub fn push_new_values_into_existing_key_value_pair(
        &mut self,
        mut new_vec: Vec<String>,
        key: &str,
    ) -> Option<usize> {
        if self.key_exists(key.to_string()) {
            if let ValueType::ListType(current_value) =
                self.get_live_item(key).unwrap().get_value().to_owned()
            {
                let mut old_vector = current_value;
                new_vec.append(&mut old_vector);
                let vec_len = new_vec.len();
                let vt_item =
                    ValueTimeItem::new_now(ValueType::ListType(new_vec), KeyAccessTime::Persistent);
                self.add(key.to_string(), vt_item);
                Some(vec_len)
            } else {
                None
            }
        } else {
            Some(0)
        }
    }

    pub fn push_new_values_into_existing_or_non_existing_key_value_pair(
        &mut self,
        mut new_vec: Vec<String>,
        key: &str,
    ) -> Option<usize> {
        if self.key_exists(key.to_string()) {
            if let ValueType::ListType(current_value) =
                self.get_live_item(key).unwrap().get_value().to_owned()
            {
                let mut old_vector = current_value;
                new_vec.append(&mut old_vector);
                let vec_len = new_vec.len();
                let vt_item =
                    ValueTimeItem::new_now(ValueType::ListType(new_vec), KeyAccessTime::Persistent);
                self.add(key.to_string(), vt_item);
                Some(vec_len)
            } else {
                None
            }
        } else {
            let vec_len = new_vec.len();
            let vt_item =
                ValueTimeItem::new_now(ValueType::ListType(new_vec), KeyAccessTime::Persistent);
            self.add(key.to_string(), vt_item);
            Some(vec_len)
        }
    }

    pub fn get_values_from_list_value_type(
        &mut self,
        key: &str,
        lower_bound: &str,
        upper_bound: &str,
    ) -> Option<Vec<String>> {
        if self.key_exists(key.to_string()) {
            if let ValueType::ListType(current_value) =
                self.get_live_item(key).unwrap().get_value().to_owned()
            {
                let current_value_len = current_value.len() as isize;
                let mut vec_values_selected_by_index = vec![];
                let mut lb = lower_bound.parse::<isize>().unwrap();
                let mut ub = upper_bound.parse::<isize>().unwrap();

                if lb < 0 {
                    lb = current_value_len + lb + 1;
                }
                if ub < 0 {
                    ub = current_value_len + lb + 1;
                }

                if ub <= current_value_len {
                    for j in lb..ub {
                        vec_values_selected_by_index.push(current_value[j as usize].clone());
                    }
                } else {
                    for j in lb..current_value_len {
                        vec_values_selected_by_index.push(current_value[j as usize].clone());
                    }
                }
                Some(vec_values_selected_by_index)
            } else {
                None
            }
        } else {
            None
        }
    }

    ///
    /// Actualiza el valor de `last_access_time` para una key.
    ///
    /// A partir de una `key` dada se actualiza el valor de
    /// `last_access_time` con el momento en que se llama a la función.
    /// Si la `key` llegó a su timeout o la key no existe en el listado
    /// de items de la database se retorna None.
    ///
    /// # Ejemplos
    ///
    /// 1. Actualiza `last_access_time` para una key sin TTL
    ///
    ///```
    ///use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
    /// use std::thread::sleep;
    /// use std::time::Duration;
    ///
    /// // Agrego los datos en la base de datos
    /// let mut db = Database::new("dummy_db_doc_reboot.csv".to_string());
    /// db.add("altura_juan".to_string(),ValueTimeItem::new_now(
    /// ValueType::StringType("1.78".to_string()),
    /// KeyAccessTime::Persistent
    /// ));
    ///
    /// let time_before_reboot = db.get_live_item("altura_juan").unwrap().get_last_access_time().clone();
    ///
    /// println!("Antes de la actualización: {}", time_before_reboot);
    /// sleep(Duration::from_secs(2));
    ///
    /// //Reseteo el tiempo de acceso a la key: "altura_juan"
    /// let res = db.reboot_time("altura_juan".to_string());
    ///
    /// match res {
    ///     Some(item) => { assert!(item.get_last_access_time() > &time_before_reboot); }
    ///     _ => assert!(false)
    /// }
    ///
    /// let _ = std::fs::remove_file("dummy_db_doc_reboot.csv");
    /// ```
    ///
    /// 2. Actualiza `last_access_time` para una key vencida
    ///
    ///```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueType, ValueTimeItem, KeyAccessTime};
    /// use std::time::{SystemTime, Duration};
    /// use std::thread::sleep;
    ///
    /// let mut db = Database::new("dummy_db_doc_reboot2.csv".to_string());
    ///
    /// //Le pongo vencimiento en now
    /// let timeout =  SystemTime::now()
    ///  .duration_since(SystemTime::UNIX_EPOCH)
    ///   .unwrap().as_secs();
    ///
    /// db.add("altura_juan".to_string(),ValueTimeItem::new_now(
    /// ValueType::StringType("1.78".to_string()),
    /// KeyAccessTime::Volatile(timeout)));
    ///
    /// //Dejo vencer la key
    /// sleep(Duration::from_secs(1));
    ///
    /// //Seteo last_access_time
    /// let res = db.reboot_time("altura_juan".to_string());
    ///
    /// match res {
    ///     None => assert!(true),
    ///     _ => assert!(false)
    /// }
    ///
    /// let _ = std::fs::remove_file("dummy_db_doc_reboot2.csv");
    ///

    pub fn reboot_time(&mut self, key: String) -> Option<&mut ValueTimeItem> {
        let mut item = self.get_mut_live_item(&key);
        if let Some(item) = &mut item {
            item.reboot_last_access_time();
        }
        item
    }

    ///Devuelve el tipo de dato del value
    pub fn get_type_of_value(&self, key: String) -> String {
        self.items.get(&key).unwrap().get_value_type()
    }

    /// Copia el valor almacenado en una clave origen a una clave destino.
    ///
    /// Si el parámetro `replace` es true, entonces reemplaza el valor almacenado en la clave destino
    /// por el valor de la clave origen. Si es false y la clave destino ya existe, devuelve None.
    /// Si la clave destino no existe, se crea.
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, KeyAccessTime, ValueType};
    ///
    /// let mut db = Database::new("dummy_db_copy.csv".to_string());
    /// db.add("dolly".to_string(),ValueTimeItem::new_now(
    /// ValueType::StringType("sheep".to_string()),
    /// KeyAccessTime::Persistent
    /// ));
    /// db.copy(String::from("dolly"), String::from("clone"), true);
    ///
    /// let copied = db.get_live_item("clone").unwrap();
    /// if let ValueType::StringType(str) = copied.get_value() {
    ///    assert_eq!(str, &String::from("sheep"));
    /// }
    ///
    /// let _ = std::fs::remove_file("dummy_db_copy.csv");
    /// ```
    pub fn copy(&mut self, source: String, destination: String, replace: bool) -> Option<()> {
        return if let Some(source_item) = self.get_live_item(&source) {
            let new_value = source_item.get_copy_of_value();
            let timeout = source_item.get_copy_of_timeout();
            match self.get_mut_live_item(&destination) {
                Some(dest) => {
                    if replace {
                        dest.set_value(new_value);
                        Some(())
                    } else {
                        None
                    }
                }
                None => {
                    // Si no existe la key, la creo.
                    self.add(destination, ValueTimeItem::new_now(new_value, timeout));
                    Some(())
                }
            }
        } else {
            None
        };
    }

    pub fn persist(&mut self, key: String) -> bool {
        match self.get_mut_live_item(&key) {
            Some(item) => item.make_persistent(),
            None => false,
        }
    }

    /// Renombra una clave.
    ///
    /// Si la clave existe, se renombra y sobreescribe con una copia del valor almacenado.
    /// En ese caso, el método devuelve true, sino false.
    ///
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
    ///
    /// // Agrego los datos en la base de datos
    /// let mut db = Database::new("dummy_db_rename.csv".to_string());
    /// db.add("dolly".to_string(),ValueTimeItem::new_now(
    /// ValueType::StringType("sheep".to_string()),
    /// KeyAccessTime::Persistent
    /// ));
    /// let renamed = db.rename_key(String::from("dolly"), String::from("newdolly"));
    ///
    /// assert_eq!(renamed, true);
    ///
    /// let _ = std::fs::remove_file("dummy_db_rename.csv");
    /// ```
    pub fn rename_key(&mut self, current_key: String, new_key: String) -> bool {
        let item = self.get_mut_live_item(&current_key);
        if let Some(item) = item {
            let item_value = item.get_copy_of_value();
            let item_time = item.get_copy_of_timeout();
            self.add(new_key, ValueTimeItem::new_now(item_value, item_time));
            true
        } else {
            false
        }
    }

    /// Concatena un string al string almacenado en la clave especificada.
    ///
    /// Si la clave existe y el valor almacenado es de tipo String, concatena el string especificado
    /// al final del existente. Si la clave no existe, se crea con el string como valor.
    /// Devuelve la longitud del nuevo string almacenado.
    ///
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
    ///
    /// // Agrego los datos en la base de datos
    /// let mut db = Database::new("dummy_db_append.csv".to_string());
    /// db.add("field".to_string(),ValueTimeItem::new_now(
    /// ValueType::StringType("first".to_string()),
    /// KeyAccessTime::Persistent
    /// ));
    /// let len = db.append_string("field", "name");
    ///
    /// assert_eq!(len, 9);
    ///
    /// let _ = std::fs::remove_file("dummy_db_append.csv");
    /// ```
    pub fn append_string(&mut self, key: &str, string: &str) -> usize {
        match self.get_mut_live_item(&key.to_string()) {
            Some(item) => {
                if let ValueType::StringType(old_value) = item.get_copy_of_value() {
                    let len = old_value.len() + string.len();
                    let new_value = ValueType::StringType(old_value + string);
                    item.set_value(new_value);
                    len
                } else {
                    0
                }
            }
            None => {
                self.items.insert(
                    key.to_string(),
                    ValueTimeItem::new_now(
                        ValueType::StringType(string.to_string()),
                        KeyAccessTime::Persistent,
                    ),
                );
                string.len()
            }
        }
    }

    /// Decrementa el valor del numero almacenado en la clave especificada.
    ///
    /// Si la clave existe y el valor almacenado es de tipo String, decrementa el valor almacenado
    /// en las unidades especificadas. Si la clave no existe, se crea con el valor 0 y luego se realiza la operación.
    /// Si el valor almacenado en la clave no es de tipo String o si no puede ser representado como un numero entero,
    /// devuelve error, sino devuelve el nuevo valor resultado de la operación.
    ///
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
    ///
    /// // Agrego los datos en la base de datos
    /// let mut db = Database::new("dummy_db_decrement.csv".to_string());
    /// db.add("edad".to_string(),ValueTimeItem::new_now(
    /// ValueType::StringType("25".to_string()),
    /// KeyAccessTime::Persistent
    /// ));
    /// let nueva_edad = db.decrement_key_by("edad", 10).unwrap();
    ///
    /// assert_eq!(nueva_edad, 15);
    ///
    /// let _ = std::fs::remove_file("dummy_db_decrement.csv");
    /// ```
    pub fn decrement_key_by(&mut self, key: &str, decr: i64) -> Result<i64, DatabaseError> {
        match self.get_mut_live_item(&key.to_string()) {
            Some(item) => {
                if let ValueType::StringType(str) = item.get_copy_of_value() {
                    if let Ok(str_as_number) = str.parse::<i64>() {
                        let new_value = ValueType::StringType((str_as_number - decr).to_string());
                        item.set_value(new_value);
                        Ok(str_as_number - decr)
                    } else {
                        Err(DatabaseError::InvalidParameter(String::from(
                            "Decrement value can't be represented as integer",
                        )))
                    }
                } else {
                    Err(DatabaseError::InvalidValueType(format!(
                        "Invalid value type. Expected: String. Got: {}",
                        item.get_value_type()
                    )))
                }
            }
            None => {
                let new_value = 0 - decr;
                self.items.insert(
                    key.to_string(),
                    ValueTimeItem::new_now(
                        ValueType::StringType(new_value.to_string()),
                        KeyAccessTime::Persistent,
                    ),
                );
                Ok(new_value)
            }
        }
    }

    /// Incrementa el valor del numero almacenado en la clave especificada.
    ///
    /// Si la clave existe y el valor almacenado es de tipo String, incrementa el valor almacenado
    /// en las unidades especificadas. Si la clave no existe, se crea con el valor 0 y luego se realiza la operación.
    /// Si el valor almacenado en la clave no es de tipo String o si no puede ser representado como un numero entero,
    /// devuelve error, sino devuelve el nuevo valor resultado de la operación.
    ///
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
    ///
    /// // Agrego los datos en la base de datos
    /// let mut db = Database::new("dummy_db_increment.csv".to_string());
    /// db.add("edad".to_string(),ValueTimeItem::new_now(
    /// ValueType::StringType("25".to_string()),
    /// KeyAccessTime::Persistent
    /// ));
    /// let nueva_edad = db.increment_key_by("edad", 10).unwrap();
    ///
    /// assert_eq!(nueva_edad, 35);
    ///
    /// let _ = std::fs::remove_file("dummy_db_increment.csv");
    /// ```
    pub fn increment_key_by(&mut self, key: &str, incr: i64) -> Result<i64, DatabaseError> {
        if let Some(item) = self.get_mut_live_item(&key.to_string()) {
            if let ValueType::StringType(str) = item.get_copy_of_value() {
                if let Ok(str_as_number) = str.parse::<i64>() {
                    let new_value = ValueType::StringType((str_as_number + incr).to_string());
                    item.set_value(new_value);
                    Ok(str_as_number + incr)
                } else {
                    Err(DatabaseError::InvalidParameter(String::from(
                        "Increment value can't be represented as integer",
                    )))
                }
            } else {
                return Err(DatabaseError::InvalidValueType(format!(
                    "Invalid value type. Expected: String. Got: {}",
                    item.get_value_type()
                )));
            }
        } else {
            let new_value = incr;
            self.items.insert(
                key.to_string(),
                ValueTimeItem::new_now(
                    ValueType::StringType(new_value.to_string()),
                    KeyAccessTime::Volatile(3423423),
                ),
            );
            Ok(new_value)
        }
    }

    /// Devuelve el valor almacenado en `key` si dicho valor es de tipo String.
    ///
    /// Retorna `None` si la clave no existe o si el valor almacenado no es de tipo String.
    ///
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
    ///
    /// let mut db = Database::new("dummy_db_get_string".to_string());
    /// let vt_1 = ValueTimeItem::new_now(
    ///    ValueType::StringType("hola".to_string()),
    ///    KeyAccessTime::Persistent,
    /// );
    /// let vt_2 = ValueTimeItem::new_now(
    ///    ValueType::ListType(vec!["hola".to_string(), "chau".to_string()]),
    ///    KeyAccessTime::Persistent,
    /// );
    /// db.add("saludo".to_string(), vt_1);
    /// db.add("saludo_despido".to_string(), vt_2);
    /// let aux = db.get_string_value_by_key("saludo").unwrap();
    /// assert_eq!(aux, String::from("hola"));
    /// let aux = db.get_string_value_by_key("saludo_despido");
    /// assert!(aux.is_none());
    ///
    /// let _ = std::fs::remove_file("dummy_db_get_string.csv");
    /// ```
    pub fn get_string_value_by_key(&mut self, key: &str) -> Option<String> {
        let item = self.get_live_item(&key.to_string());
        if let Some(item) = item {
            let value = item.get_copy_of_value();
            if let ValueType::StringType(str) = value {
                return Some(str);
            }
        }
        None
    }

    /// Retorna la longitud del string almacenado en `key`.
    ///
    /// Si la clave existe y el valor almacenado es de tipo String, retorna la longitud del mismo.
    /// Si la clave no existe, retorna 0. Retorna None si el valor almacenado no es de tipo String.
    ///
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
    ///
    /// let mut db = Database::new("dummy_db_strlen.csv".to_string());
    /// db.add("mykey".to_string(),ValueTimeItem::new_now(
    /// ValueType::StringType("myvalue".to_string()),
    /// KeyAccessTime::Persistent
    /// ));
    /// let len = db.get_strlen_by_key("mykey").unwrap();
    ///
    /// assert_eq!(len, 7);
    ///
    /// let _ = std::fs::remove_file("dummy_db_strlen.csv");
    /// ```
    pub fn get_strlen_by_key(&mut self, key: &str) -> Option<usize> {
        //agregar tests
        let item = self.get_live_item(&key.to_string());
        if let Some(item) = item {
            let value = item.get_copy_of_value();
            if let ValueType::StringType(str) = value {
                Some(str.len())
            } else {
                None
            }
        } else {
            Some(0)
        }
    }

    /// Obtiene el valor almacenado en `key` y luego elimina la clave.
    ///
    /// Si la clave existe y el valor almacenado es de tipo String, obtiene el valor y luego elimina la clave.
    /// Retorna el valor obtenido. Si la clave no existe, retorna error `MissingKey`. Si el valor almacenado no es de tipo String
    /// retorna error de tipo `InvalidValueType`.
    ///
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
    ///
    /// let mut db = Database::new("dummy_db_getdel.csv".to_string());
    /// db.add("mykey".to_string(),ValueTimeItem::new_now(
    /// ValueType::StringType("myvalue".to_string()),
    /// KeyAccessTime::Persistent
    /// ));
    /// let deleted_value = db.getdel_value_by_key("mykey").unwrap();
    ///
    /// assert_eq!(deleted_value, String::from("myvalue"));
    ///
    /// let _ = std::fs::remove_file("dummy_db_getdel.csv");
    /// ```
    pub fn getdel_value_by_key(&mut self, key: &str) -> Result<String, DatabaseError> {
        //agregar tests
        let item = self.get_live_item(&key.to_string());
        if let Some(item) = item {
            let value = item.get_copy_of_value();
            if let ValueType::StringType(str) = value {
                self.delete_key(key.to_string());
                Ok(str)
            } else {
                Err(DatabaseError::InvalidValueType(format!(
                    "Invalid value type. Expected: String. Got: {}",
                    item.get_value_type()
                )))
            }
        } else {
            Err(DatabaseError::MissingKey())
        }
    }

    /// Obtiene el valor almacenado en `key` y luego actualiza la clave con `new_value`.
    ///
    /// Si la clave existe y el valor almacenado es de tipo String, obtiene el valor y luego actualiza la clave.
    /// Retorna el valor obtenido. Si la clave no existe, retorna error `MissingKey`. Si el valor almacenado no es de tipo String
    /// retorna error de tipo `InvalidValueType`.
    ///
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
    ///
    /// let mut db = Database::new("dummy_db_getset.csv".to_string());
    /// db.add("mykey".to_string(),ValueTimeItem::new_now(
    /// ValueType::StringType("myvalue".to_string()),
    /// KeyAccessTime::Persistent
    /// ));
    /// let deleted_value = db.getset_value_by_key("mykey", "newvalue").unwrap();
    ///
    /// assert_eq!(deleted_value, String::from("myvalue"));
    ///
    /// let _ = std::fs::remove_file("dummy_db_getset.csv");
    /// ```
    pub fn getset_value_by_key(
        &mut self,
        key: &str,
        new_value: &str,
    ) -> Result<String, DatabaseError> {
        //agregar tests
        let item_optional = self.get_mut_live_item(&key.to_string());
        if let Some(item) = item_optional {
            let value = item.get_copy_of_value();
            if let ValueType::StringType(str) = value {
                item.set_value(ValueType::StringType(new_value.to_string()));
                Ok(str)
            } else {
                Err(DatabaseError::InvalidValueType(format!(
                    "Invalid value type. Expected: String. Got: {}",
                    item.get_value_type()
                )))
            }
        } else {
            Err(DatabaseError::MissingKey())
        }
    }

    /// Retorna la cantidad de elementos almacenados en el Set de la clave especificada.
    ///
    /// Si la clave no existe o el valor no es de tipo Set, devuelve 0.
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
    /// use std::collections::HashSet;
    ///
    /// let mut db = Database::new("dummy_db_setlen.csv".to_string());
    /// let mut set = HashSet::new();
    /// set.insert("25".to_string());
    /// set.insert("40".to_string());
    /// let vt = ValueTimeItem::new_now(ValueType::SetType(set), KeyAccessTime::Persistent);
    /// db.add("edades".to_string(), vt);
    ///
    /// let len = db.get_len_of_set("edades");
    ///
    /// assert_eq!(len, 2);
    ///
    /// let _ = std::fs::remove_file("dummy_db_setlen.csv");
    /// ```
    pub fn get_len_of_set(&mut self, key: &str) -> usize {
        let item = self.get_live_item(key);
        match item {
            Some(item) => {
                if let ValueType::SetType(item) = item.get_value() {
                    item.len()
                } else {
                    0
                }
            }
            None => 0,
        }
    }

    pub fn is_member_of_set(&mut self, key: &str, member: &str) -> usize {
        let item = self.get_live_item(key);
        if let Some(item) = item {
            if let ValueType::SetType(item) = item.get_value() {
                if item.get(member).is_some() {
                    return 1;
                }
            }
        }
        0
    }

    pub fn get_members_of_set(&mut self, key: &str) -> Vec<&String> {
        let item = self.get_live_item(key);
        let mut members = Vec::new();
        if let Some(item) = item {
            if let ValueType::SetType(item) = item.get_value() {
                item.iter().for_each(|member| {
                    members.push(member);
                });
            }
        }
        members
    }

    pub fn remove_member_from_set(&mut self, key: &str, member: &str) -> Option<bool> {
        //si el miembro no pertenece al set, lo ignoro
        //si la clave no existe, devuelvo 0 (false)
        //empty set -> devuelve 0 (false)
        //si no es tipo Set -> devuelve error (None)
        let item = self.get_live_item(key);
        match item {
            Some(item) => {
                if let ValueType::SetType(mut item) = item.get_copy_of_value() {
                    return Some(item.remove(member));
                }
            }
            None => {
                return Some(false);
            }
        }
        None
    }

    // falta opcion get
    // agregar tests unitarios (set new key, set existing key, set key timeout x2, set key set_if x2, set key timeout set_if x2, set key !set_if x2, set key timeout !set_if x2)
    pub fn set_string(
        &mut self,
        key: &str,
        value: &str,
        timeout: (&String, Option<&String>),
        set_if_exists: Option<&String>,
        _get: Option<&String>,
    ) -> bool {
        let mut set_if_non_existing = false;
        let mut set_if_existing = false;

        if let Some(exists) = set_if_exists {
            if exists == "nx" {
                set_if_non_existing = true;
            } else if exists == "xx" {
                set_if_existing = true;
            }
        }

        let expire_at = self.get_expire_at(timeout);
        let item = self.get_mut_live_item(key);
        match item {
            Some(item) => {
                if set_if_existing || !set_if_non_existing {
                    item.set_value(ValueType::StringType(value.to_string()));
                    if expire_at != 0 {
                        item.set_timeout(KeyAccessTime::Volatile(expire_at));
                    }
                    return true;
                }
            }
            None => {
                if set_if_non_existing || !set_if_existing {
                    let value = ValueType::StringType(value.to_string());
                    let mut time = KeyAccessTime::Persistent;
                    if expire_at != 0 {
                        time = KeyAccessTime::Volatile(expire_at);
                    }
                    let new_item = ValueTimeItem::new_now(value, time);
                    self.add(key.to_string(), new_item);
                    return true;
                }
            }
        }
        false
    }

    fn get_expire_at(&mut self, timeout: (&String, Option<&String>)) -> u64 {
        let mut expire_at = 0;
        match timeout.0.as_str() {
            "ex" => {
                if let Some(ex) = timeout.1 {
                    expire_at = ex.parse::<u64>().unwrap();
                }
            }
            "px" => {
                if let Some(px) = timeout.1 {
                    expire_at = px.parse::<u64>().unwrap() / 1000;
                }
            }
            "exat" => {
                if let Some(exat) = timeout.1 {
                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap();
                    expire_at = exat.parse::<u64>().unwrap() + now.as_secs();
                }
            }
            "pxat" => {
                if let Some(pxat) = timeout.1 {
                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap();
                    expire_at = pxat.parse::<u64>().unwrap() / 1000 + now.as_secs();
                }
            }
            _ => {}
        }
        expire_at
    }

    /// Elimina y retorna los primeros elementos de la lista almacenada en `key`.
    ///
    /// Por defecto, elimina el primer elemento de la lista. Si se le pasa el parámetro opcional `count`, elimina
    /// los primeros `count` elementos.
    /// Si la clave no existe, retorna `None`. Si existe, retorna una lista con los elementos eliminados.
    /// # Examples
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime};
    ///
    /// let mut db = Database::new("dummy_db_pop.csv".to_string());
    /// let mut list = vec![String::from("argentina"), String::from("brasil"), String::from("chile"), String::from("uruguay")];
    /// let vt = ValueTimeItem::new_now(ValueType::ListType(list), KeyAccessTime::Persistent);
    /// db.add("paises".to_string(), vt);
    ///
    /// let removed = db.pop_elements_from_list("paises", 1).unwrap();
    /// assert_eq!(removed, vec![String::from("argentina")]);
    ///
    /// let removed = db.pop_elements_from_list("paises", 2).unwrap();
    /// assert_eq!(removed, vec![String::from("brasil"), String::from("chile")]);
    ///
    /// let _ = std::fs::remove_file("dummy_db_pop.csv");
    /// ```
    pub fn pop_elements_from_list(&mut self, key: &str, count: usize) -> Option<Vec<String>> {
        let mut popped_elements = Vec::new();
        if let Some(item) = self.get_mut_live_item(key) {
            if let ValueType::ListType(mut list) = item.get_copy_of_value() {
                popped_elements = list.drain(..count).collect(); //validar count < len
                item.set_value(ValueType::ListType(list));
            }
        } else {
            return None;
        }
        Some(popped_elements)
    }

    /* Si el servidor se reinicia se deben cargar los items del file */
    pub fn load_items(&mut self) {
        if let Ok(lines) = Database::read_lines(self.dbfilename.to_string()) {
            for line in lines {
                if let Ok(kvi_serialized) = line {
                    let kvis = KeyValueItemSerialized::new(kvi_serialized);
                    let item = kvis.transform_to_item();
                    self.items.insert(item.0, item.1);
                } else {
                    panic!("Error al leer línea del archivo:");
                }
            }
        } else {
            panic!("Error al leer el archivo dump");
        }
    }

    fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where
        P: AsRef<Path>,
    {
        let path = Path::new(filename.as_ref());
        let file = File::open(&path);
        match file {
            Ok(_) => Ok(io::BufReader::new(file.unwrap()).lines()),
            Err(_) => {
                let mut _file = File::create(&path)?; //Lo crea en write-only mode.
                let path = Path::new(filename.as_ref());
                let file_op = File::open(&path);
                Ok(io::BufReader::new(file_op.unwrap()).lines())
            }
        }
    }
    /*
      Guarda cada item que tiene en memoria, en el formato adecuado para la serialización.
      Formato: key;last_access_time;timeout;type;value1,value,2
    */
    pub fn save_items_to_file(&self) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create_new(false)
            .open(self.dbfilename.to_string())
            .unwrap();

        for kvi in &self.items {
            let kvi_type = match kvi.1.get_value() {
                ValueType::StringType(_) => "string",
                ValueType::SetType(_) => "set",
                ValueType::ListType(_) => "list",
            };
            writeln!(
                file,
                "{};{};{};{};{}",
                kvi.0,
                kvi.1.get_last_access_time().to_string(),
                kvi.1.get_timeout().to_string(),
                kvi_type,
                kvi.1.get_value().to_string()
            )
            .unwrap();
        }
    }

    /// devuelve todos los valores almacenados en todas las claves en orden aleatorio
    pub fn _get_all_values(&self) -> Vec<ValueType> {
        let mut all_values = Vec::new();
        let values_vector = &self.items;
        for value in values_vector.values() {
            let only_value = value.get_copy_of_value();
            all_values.push(only_value);
        }
        all_values
    }

    /// devuelve la cantidad de claves almacenadas en la base de datos
    pub fn get_size(&self) -> usize {
        self.items.len()
    }

    /// permite eliminar una clave y su valor asociado
    pub fn delete_key(&mut self, key: String) -> bool {
        matches!(self.items.remove(&key), Some(_key))
    }

    /// le setea un timestamp de expiración a una determinada key
    /// Si la key no existe, devuelve false. Si el update fue correctamente generado devuelve true.
    pub fn expire_key(&mut self, key: &str, timeout: &str) -> bool {
        let kvi = self.get_mut_live_item(key);
        match kvi {
            Some(k) => k.set_timeout(KeyAccessTime::Volatile(u64::from_str(timeout).unwrap())),
            None => false,
        }
    }
}

//--------------------------------------------------------------------------------------------------
//--------------------------------------------------------------------------------------------------
//------------------------------------UNIT TESTS----------------------------------------------------
//--------------------------------------------------------------------------------------------------
//--------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::key_value_item::{KeyAccessTime, ValueType};
    use std::io::BufReader;
    use std::time::SystemTime;

    #[test]
    fn test_00_filter_keys_by_pattern() {
        let mut db = Database::new(String::from("./src/dummy_00.txt"));

        let vt_1 = ValueTimeItem::new_now(
            ValueType::StringType("valor_1".to_string()),
            KeyAccessTime::Persistent,
        );
        let vt_2 = ValueTimeItem::new_now(
            ValueType::StringType("valor_2".to_string()),
            KeyAccessTime::Persistent,
        );
        let vt_3 = ValueTimeItem::new_now(
            ValueType::StringType("valor_3".to_string()),
            KeyAccessTime::Persistent,
        );
        let vt_4 = ValueTimeItem::new_now(
            ValueType::StringType("valor_4".to_string()),
            KeyAccessTime::Persistent,
        );

        db.items.insert("weight_bananas".to_string(), vt_1);
        db.items.insert("apples_weight".to_string(), vt_2);
        db.items
            .insert("deliciosos_kiwi_weight_baratos".to_string(), vt_3);
        db.items.insert("banana_weight".to_string(), vt_4);

        // db.get_values_of_external_keys_that_match_a_pattern("banana");
        let vec_filtered = db.get_keys_that_match_pattern_sin_regex("weight".to_string());
        assert_eq!(vec_filtered.len(), 4);
        let _ = std::fs::remove_file("./src/dummy_00.txt");
    }

    #[test]
    fn test_01_empty_database_returns_cero() {
        let db = Database {
            dbfilename: "file".to_string(),
            items: HashMap::new(),
        };

        assert_eq!(db.get_size(), 0);
    }

    #[test]
    fn test_02_database_copies_value_to_new_key() {
        let mut db = Database::new(String::from("./src/dummy.txt"));
        db.add(
            "clave_1".to_string(),
            ValueTimeItem::new_now(
                ValueType::StringType("valor_1".to_string()),
                KeyAccessTime::Persistent,
            ),
        );

        let source = String::from("clave_1");
        let destination = String::from("clone");
        assert_eq!(db.copy(source, destination, false).unwrap(), ());

        let new_item = db.get_live_item("clone").unwrap();
        if let ValueType::StringType(str) = new_item.get_value() {
            assert_eq!(str, &String::from("valor_1"));
        }
        let _ = std::fs::remove_file("./src/dummy.txt");
    }

    #[test]
    fn test_03_database_copy_replaces_key_with_new_value() {
        let mut db = Database::new(String::from("./src/dummy2.txt"));
        db.add(
            "clave_1".to_string(),
            ValueTimeItem::new_now(
                ValueType::StringType("valor_1".to_string()),
                KeyAccessTime::Persistent,
            ),
        );
        db.add(
            "clave_2".to_string(),
            ValueTimeItem::new_now(
                ValueType::StringType("valor_2".to_string()),
                KeyAccessTime::Persistent,
            ),
        );

        let source = String::from("clave_1");
        let destination = String::from("clone");
        assert_eq!(db.copy(source, destination, false).unwrap(), ());

        let new_item = db.get_live_item("clone").unwrap();
        if let ValueType::StringType(str) = new_item.get_value() {
            assert_eq!(str, &String::from("valor_1"));
        }

        let source = String::from("clave_2");
        let destination = String::from("clone");
        assert_eq!(db.copy(source, destination, true).unwrap(), ());

        let new_item = db.get_live_item("clone").unwrap();
        if let ValueType::StringType(str) = new_item.get_value() {
            assert_eq!(str, &String::from("valor_2"));
        }
        let _ = std::fs::remove_file("./src/dummy2.txt");
    }

    #[test]
    fn test_04_clean_items_deletes_all_items() {
        let mut db = Database::new(String::from("./src/database1.txt"));
        db.clean_items();
        assert_eq!(db.get_size(), 0);
        std::fs::remove_file("./src/database1.txt").unwrap();
    }

    #[test]
    fn test_05_deletes_an_item_succesfully() {
        let mut db = Database::new("file2".to_string());

        let vt_1 = ValueTimeItem::new_now(
            ValueType::StringType("valor_1".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_2 = ValueTimeItem::new_now(
            ValueType::StringType("valor_2".to_string()),
            KeyAccessTime::Volatile(0),
        );
        db.items.insert("weight_bananas".to_string(), vt_1);
        db.items.insert("apples_weight".to_string(), vt_2);

        db.delete_key("apples_weight".to_string());

        assert_eq!(db.get_size(), 1);
        std::fs::remove_file("file2").unwrap();
    }

    #[test]
    fn test_06_persist_changes_type_of_access_time() {
        use crate::domain::entities::key_value_item::KeyAccessTime;
        let mut db = Database::new("file".to_string());

        let vt_1 = ValueTimeItem::new_now(
            ValueType::StringType("valor_1".to_string()),
            KeyAccessTime::Volatile(1825601548),
        );
        let vt_2 = ValueTimeItem::new_now(
            ValueType::StringType("valor_2".to_string()),
            KeyAccessTime::Volatile(1825601548),
        );
        db.items.insert("weight_bananas".to_string(), vt_1);
        db.items.insert("apples_weight".to_string(), vt_2);
        //--------
        let _res = db.persist("weight_bananas".to_string());

        let item = db.items.get(&"weight_bananas".to_string()).unwrap();
        match *item.get_timeout() {
            KeyAccessTime::Persistent => assert!(true),
            KeyAccessTime::Volatile(_tmt) => assert!(false),
        }

        std::fs::remove_file("file").unwrap();
    }

    #[test]
    fn test_07_add_item() {
        let mut db = Database {
            dbfilename: "file".to_string(),
            items: HashMap::new(),
        };
        db.add(
            String::from("nueva_key"),
            ValueTimeItem::new_now(
                ValueType::StringType(String::from("222")),
                KeyAccessTime::Persistent,
            ),
        );

        assert_eq!(
            db.items.get("nueva_key").unwrap().get_value().to_string(),
            String::from("222")
        );
        assert_eq!(db.items.len(), 1)
    }

    #[test]
    fn test_08_delete_item() {
        let mut db = Database {
            dbfilename: "file".to_string(),
            items: HashMap::new(),
        };
        db.items.insert(
            String::from("nueva_key"),
            ValueTimeItem::new_now(
                ValueType::StringType(String::from("222")),
                KeyAccessTime::Persistent,
            ),
        );

        assert_eq!(db.items.len(), 1);
        db.delete_key(String::from("nueva_key"));
        assert_eq!(db.items.len(), 0);
    }

    #[test]
    fn test_09_filename_is_correct() {
        let db = Database {
            dbfilename: "file".to_string(),
            items: HashMap::new(),
        };
        assert_eq!(db._get_filename(), "file".to_string());
    }

    #[test]
    fn test_10_load_items_from_file() {
        let mut file = File::create("file_5".to_string()).expect("Unable to open");
        file.write_all(b"124key;1623433670;1623433677;string;value2\n")
            .unwrap();

        let db = Database::new("file_5".to_string());
        assert_eq!(db.items.len(), 1);
        let mut iter = db.items.iter();
        let kvi = iter.next().unwrap();

        assert_eq!(kvi.0, "124key");
        assert_eq!(kvi.1.get_value().to_string(), String::from("value2"));
        match kvi.1.get_timeout() {
            KeyAccessTime::Volatile(1623433677) => assert!(true),
            _ => assert!(false),
        }
        let _ = std::fs::remove_file("file_5");
    }

    #[test]
    fn test_11_create_database_file() {
        assert!(!std::path::Path::new("new_file").exists());
        let _db = Database::new("new_file".to_string());
        assert!(std::path::Path::new("new_file").exists());
        let _ = std::fs::remove_file("new_file");
    }

    #[test]
    fn test_12_save_items_to_file() {
        let mut db = Database::new("file".to_string());

        let list = vec![
            "un_item_string".to_string(),
            "segundo_item_list_string".to_string(),
        ];

        db.items.insert(
            "clave_2".to_string(),
            ValueTimeItem::new_now(ValueType::ListType(list), KeyAccessTime::Volatile(1231230)),
        );
        let last_access_time = db
            .items
            .get("clave_2")
            .unwrap()
            .get_last_access_time()
            .to_string();

        db.save_items_to_file();

        let file = File::open(&db.dbfilename);
        let reader = BufReader::new(file.unwrap());
        let mut it = reader.lines();

        let line_serialized = "clave_2;".to_owned()
            + last_access_time.as_str()
            + ";1231230;list;un_item_string,segundo_item_list_string";

        match it.next().unwrap() {
            Ok(t) => assert_eq!(t, line_serialized),
            _ => assert!(false),
        }

        let _ = std::fs::remove_file("file");
    }

    #[test]
    fn test_13_size_in_memory_is_correct() {
        let mut db = Database::new("file1".to_string());

        let vt_1 = ValueTimeItem::new_now(
            ValueType::StringType("valor_1".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_2 = ValueTimeItem::new_now(
            ValueType::StringType("valor_2".to_string()),
            KeyAccessTime::Volatile(0),
        );
        db.items.insert("weight_bananas".to_string(), vt_1);
        db.items.insert("apples_weight".to_string(), vt_2);
        std::fs::remove_file("file1").unwrap();
    }

    #[test]
    fn test_14_persist_changes_type_of_access_time() {
        use crate::domain::entities::key_value_item::KeyAccessTime;
        let mut db = Database::new(String::from("./src/dummy_persist.txt"));
        let _res = db.items.insert(
            "clave_1".to_string(),
            ValueTimeItem::new_now(
                ValueType::StringType("value".to_string()),
                KeyAccessTime::Persistent,
            ),
        );

        let item = db.items.get("clave_1").unwrap();
        match *item.get_timeout() {
            KeyAccessTime::Persistent => assert!(true),
            KeyAccessTime::Volatile(_tmt) => assert!(false),
        }
        std::fs::remove_file("./src/dummy_persist.txt".to_string()).unwrap();
    }

    #[test]
    fn test_15_append_adds_string_to_end_of_existing_value() {
        let mut db = Database::new(String::from("./src/dummy_appends_2.txt"));
        let _res = db.items.insert(
            "mykey".to_string(),
            ValueTimeItem::new_now(
                ValueType::StringType("Hello".to_string()),
                KeyAccessTime::Persistent,
            ),
        );

        let len = db.append_string(&"mykey".to_string(), &" World".to_string());
        assert_eq!(len, 11);
        std::fs::remove_file("./src/dummy_appends_2.txt".to_string()).unwrap();
    }

    #[test]
    fn test_16_append_adds_string_to_new_value() {
        let mut db = Database::new(String::from("./src/dummy_appends_1.txt"));

        let len = db.append_string(&"mykey".to_string(), &" World".to_string());
        assert_eq!(len, 6);
        std::fs::remove_file("./src/dummy_appends_1.txt".to_string()).unwrap();
    }

    #[test]
    fn test_17_decr_key_to_existing_key() {
        let mut db = Database::new(String::from("./src/dummy_decr_1.txt"));
        let _res = db.items.insert(
            "mykey".to_string(),
            ValueTimeItem::new_now(
                ValueType::StringType("10".to_string()),
                KeyAccessTime::Persistent,
            ),
        );

        let res = db.decrement_key_by(&"mykey".to_string(), 3).unwrap();
        assert_eq!(res, 7);
        std::fs::remove_file("./src/dummy_decr_1.txt".to_string()).unwrap();
    }

    #[test]
    fn test_18_decr_by_to_new_key() {
        let mut db = Database::new(String::from("./src/dummy_decr.txt"));

        let res = db.decrement_key_by(&"mykey".to_string(), 3).unwrap();
        assert_eq!(res, -3);
        std::fs::remove_file("./src/dummy_decr.txt".to_string()).unwrap();
    }

    #[test]
    fn test_19_decr_by_to_invalid_string_value() {
        let mut db = Database::new(String::from("./src/dummy_decr_2.txt"));
        let _res = db.items.insert(
            "mykey".to_string(),
            ValueTimeItem::new_now(
                ValueType::StringType("Hello".to_string()),
                KeyAccessTime::Persistent,
            ),
        );

        let res = db.decrement_key_by(&"mykey".to_string(), 3);
        assert!(res.is_err());
        let _ = std::fs::remove_file("./src/dummy_decr_2.txt".to_string());
    }

    #[test]
    fn test_20_se_obtienen_valores_de_claves_externas_a_partir_de_un_patron_y_una_lista_de_elementos(
    ) {
        let mut db = Database::new("file10".to_string());

        let vt_1 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_2 = ValueTimeItem::new_now(
            ValueType::StringType("2".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_3 = ValueTimeItem::new_now(
            ValueType::StringType("11".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_4 = ValueTimeItem::new_now(
            ValueType::StringType("5".to_string()),
            KeyAccessTime::Volatile(0),
        );
        db.items.insert("weight_bananas".to_string(), vt_1);
        db.items.insert("weight_apples".to_string(), vt_2);
        db.items.insert("weight_kiwi".to_string(), vt_3);
        db.items.insert("weight_pear".to_string(), vt_4);

        let pat = "weight_";
        let vec_strings = vec![
            "sandia".to_string(),
            "pear".to_string(),
            "apples".to_string(),
        ];
        let tuplas = db.get_values_of_external_keys_that_match_a_pattern(vec_strings, pat);
        let algo = tuplas.unwrap();
        println!("{:?}", algo);
        let _removed = std::fs::remove_file("file10".to_string());
    }

    #[test]
    fn test_21_se_obtienen_keys_que_contienen_patron_regex_con_signo_de_pregunta() {
        let mut db = Database::new("file11".to_string());

        let vt_1 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_2 = ValueTimeItem::new_now(
            ValueType::StringType("2".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_3 = ValueTimeItem::new_now(
            ValueType::StringType("11".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_4 = ValueTimeItem::new_now(
            ValueType::StringType("5".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_5 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_6 = ValueTimeItem::new_now(
            ValueType::StringType("2".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_7 = ValueTimeItem::new_now(
            ValueType::StringType("11".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_8 = ValueTimeItem::new_now(
            ValueType::StringType("5".to_string()),
            KeyAccessTime::Volatile(0),
        );
        db.items.insert("pablo".to_string(), vt_1);
        db.items.insert("juan".to_string(), vt_2);
        db.items.insert("mariana".to_string(), vt_3);
        db.items.insert("lucia".to_string(), vt_4);
        db.items.insert("mariano".to_string(), vt_5);
        db.items.insert("meriana".to_string(), vt_6);
        db.items.insert("miriana".to_string(), vt_7);
        db.items.insert("luciana".to_string(), vt_8);

        let pat = "m?riana";
        let matching_keys = db.get_keys_that_match_pattern(pat);
        //let algo = tuplas.unwrap();
        for key in matching_keys {
            println!("{:?}", key)
        }
        let _removed = std::fs::remove_file("file11".to_string());
    }

    #[test]
    fn test_22_se_obtienen_keys_que_contienen_patron_regex_solo_exp_entre_corchetes() {
        let mut db = Database::new("file12".to_string());

        let vt_1 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_2 = ValueTimeItem::new_now(
            ValueType::StringType("2".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_3 = ValueTimeItem::new_now(
            ValueType::StringType("11".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_4 = ValueTimeItem::new_now(
            ValueType::StringType("5".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_5 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_6 = ValueTimeItem::new_now(
            ValueType::StringType("2".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_7 = ValueTimeItem::new_now(
            ValueType::StringType("11".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_8 = ValueTimeItem::new_now(
            ValueType::StringType("5".to_string()),
            KeyAccessTime::Volatile(0),
        );
        db.items.insert("mia".to_string(), vt_1);
        db.items.insert("juan".to_string(), vt_2);
        db.items.insert("mariana".to_string(), vt_3);
        db.items.insert("lucia".to_string(), vt_4);
        db.items.insert("malala".to_string(), vt_5);
        db.items.insert("meriana".to_string(), vt_6);
        db.items.insert("miriana".to_string(), vt_7);
        db.items.insert("luciana".to_string(), vt_8);

        let pat = "m[ae]riana";
        let matching_keys = db.get_keys_that_match_pattern(pat);
        //let algo = tuplas.unwrap();
        for key in matching_keys {
            println!("{:?}", key)
        }
        let _removed = std::fs::remove_file("file12".to_string());
    }

    #[test]
    fn test_23_se_obtienen_keys_que_contienen_patron_regex_excepto_exp_entre_corchetes_tipo_1() {
        let mut db = Database::new("file13".to_string());

        let vt_1 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_2 = ValueTimeItem::new_now(
            ValueType::StringType("2".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_3 = ValueTimeItem::new_now(
            ValueType::StringType("11".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_4 = ValueTimeItem::new_now(
            ValueType::StringType("5".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_5 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_6 = ValueTimeItem::new_now(
            ValueType::StringType("2".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_7 = ValueTimeItem::new_now(
            ValueType::StringType("11".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_8 = ValueTimeItem::new_now(
            ValueType::StringType("5".to_string()),
            KeyAccessTime::Volatile(0),
        );
        db.items.insert("mia".to_string(), vt_1);
        db.items.insert("juan".to_string(), vt_2);
        db.items.insert("mariana".to_string(), vt_3);
        db.items.insert("lucia".to_string(), vt_4);
        db.items.insert("malala".to_string(), vt_5);
        db.items.insert("meriana".to_string(), vt_6);
        db.items.insert("miriana".to_string(), vt_7);
        db.items.insert("luciana".to_string(), vt_8);

        let pat = "m[^a]riana";
        let matching_keys = db.get_keys_that_match_pattern(pat);
        //let algo = tuplas.unwrap();
        for key in matching_keys {
            println!("{:?}", key)
        }
        let _removed = std::fs::remove_file("file13".to_string());
    }

    #[test]
    fn test_24_se_obtienen_keys_que_contienen_patron_regex_excepto_exp_entre_corchetes_tipo_2_rango(
    ) {
        let mut db = Database::new("file14".to_string());

        let vt_1 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_2 = ValueTimeItem::new_now(
            ValueType::StringType("2".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_3 = ValueTimeItem::new_now(
            ValueType::StringType("11".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_4 = ValueTimeItem::new_now(
            ValueType::StringType("5".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_5 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_6 = ValueTimeItem::new_now(
            ValueType::StringType("2".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_7 = ValueTimeItem::new_now(
            ValueType::StringType("11".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_8 = ValueTimeItem::new_now(
            ValueType::StringType("5".to_string()),
            KeyAccessTime::Volatile(0),
        );
        db.items.insert("mia".to_string(), vt_1);
        db.items.insert("juan".to_string(), vt_2);
        db.items.insert("mariana".to_string(), vt_3);
        db.items.insert("muriana".to_string(), vt_4);
        db.items.insert("malala".to_string(), vt_5);
        db.items.insert("meriana".to_string(), vt_6);
        db.items.insert("miriana".to_string(), vt_7);
        db.items.insert("moriana".to_string(), vt_8);

        let pat = "m[a-o]riana";
        let matching_keys = db.get_keys_that_match_pattern(pat);
        //let algo = tuplas.unwrap();
        for key in matching_keys {
            println!("{:?}", key)
        }
        let _ = std::fs::remove_file("file14".to_string());
    }

    #[test]
    fn test_25_se_obtienen_keys_que_contienen_patron_regex_asterisco() {
        let mut db = Database::new("file15".to_string());

        let vt_1 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_2 = ValueTimeItem::new_now(
            ValueType::StringType("2".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_3 = ValueTimeItem::new_now(
            ValueType::StringType("11".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_4 = ValueTimeItem::new_now(
            ValueType::StringType("5".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_5 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_6 = ValueTimeItem::new_now(
            ValueType::StringType("2".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_7 = ValueTimeItem::new_now(
            ValueType::StringType("11".to_string()),
            KeyAccessTime::Volatile(0),
        );
        let vt_8 = ValueTimeItem::new_now(
            ValueType::StringType("5".to_string()),
            KeyAccessTime::Volatile(0),
        );
        db.items.insert("mia".to_string(), vt_1);
        db.items.insert("jose".to_string(), vt_2);
        db.items.insert("mariana".to_string(), vt_3);
        db.items.insert("pedro".to_string(), vt_4);
        db.items.insert("malala".to_string(), vt_5);
        db.items.insert("meriana".to_string(), vt_6);
        db.items.insert("miriana".to_string(), vt_7);
        db.items.insert("moriana".to_string(), vt_8);

        let pat = "m*a";
        let matching_keys = db.get_keys_that_match_pattern(pat);
        //let algo = tuplas.unwrap();
        for key in matching_keys {
            println!("{:?}", key)
        }
        let _ = std::fs::remove_file("file15".to_string());
    }

    #[test]
    fn test_26_expire_key() {
        let mut db = Database::new("file100".to_string());
        let vt_1 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(1825601548),
        );
        db.items.insert("key123".to_string(), vt_1);
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let new_timeout = u64::from_str("10").unwrap() + now.as_secs();
        db.expire_key("key123", &new_timeout.to_string());
        let new_item = db.items.get("key123");
        match new_item {
            Some(vti) => {
                assert_eq!(vti.get_timeout().to_string(), new_timeout.to_string());
            }
            None => assert!(false),
        }
        let _ = std::fs::remove_file("file100".to_string());
    }

    #[test]
    fn test_22_reboot_time() {
        let mut db = Database::new("file022a".to_string());
        let vt_1 = ValueTimeItem::new(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(1925583652),
            u64::from_str("1211111").unwrap(),
        );
        db.items.insert("key123".to_string(), vt_1);
        let old_access_time = db.items.get("key123").unwrap().get_last_access_time();
        assert_eq!(old_access_time, &u64::from_str("1211111").unwrap());
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Some(vti) = db.reboot_time("key123".to_string()) {
            assert!(vti.get_last_access_time().ge(&now));
        } else {
            assert!(false)
        }

        let _ = std::fs::remove_file("file022a".to_string());
    }
    #[test]
    fn test_22_reboot_time_expired() {
        let mut db = Database::new("file022b".to_string());
        let vt_1 = ValueTimeItem::new(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(12123120),
            u64::from_str("1211111").unwrap(),
        );
        db.items.insert("key123".to_string(), vt_1);
        let old_access_time = db.items.get("key123").unwrap().get_last_access_time();
        assert_eq!(old_access_time, &u64::from_str("1211111").unwrap());

        if let None = db.reboot_time("key123".to_string()) {
            assert!(true)
        } else {
            assert!(false)
        }

        let _ = std::fs::remove_file("file022b".to_string());
    }

    #[test]
    fn test_23_expired_passive_keys() {
        let mut db = Database::new("file023".to_string());
        let vt_1 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(1625326138),
        );
        db.items.insert("key123".to_string(), vt_1);

        assert!(db.items.get("key123").is_some());
        let item_expired = db.get_live_item(&"key123".to_string());
        match item_expired {
            Some(_) => assert!(false),
            None => assert!(true),
        }
        assert!(db.items.get("key123").is_none());
        let _ = std::fs::remove_file("file023".to_string());
    }

    #[test]
    fn test_24_retrieve_live_keys() {
        let mut db = Database::new("file024".to_string());
        let vt_1 = ValueTimeItem::new_now(
            ValueType::StringType("1".to_string()),
            KeyAccessTime::Volatile(1665326138),
        );
        db.items.insert("key123".to_string(), vt_1);

        assert!(db.items.get("key123").is_some());
        let item_expired = db.get_live_item(&"key123".to_string());
        match item_expired {
            Some(_) => assert!(true),
            None => assert!(false),
        }
        assert!(db.items.get("key123").is_some());
        let _ = std::fs::remove_file("file024".to_string());
    }
}

#[test]
fn test_27_se_obtienen_las_claves_que_contienen_solo_string_values() {
    use std::collections::HashSet;

    let mut db = Database::new("file025".to_string());

    let vt_1 = ValueTimeItem::new_now(
        ValueType::StringType("hola".to_string()),
        KeyAccessTime::Persistent,
    );
    let vt_2 = ValueTimeItem::new_now(
        ValueType::StringType("chau".to_string()),
        KeyAccessTime::Persistent,
    );
    let vt_3 = ValueTimeItem::new_now(
        ValueType::ListType(vec!["hola".to_string(), "chau".to_string()]),
        KeyAccessTime::Persistent,
    );
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());
    this_set.insert("value_2".to_string());
    let vt_4 = ValueTimeItem::new_now(ValueType::SetType(this_set), KeyAccessTime::Volatile(0));
    db.add("saludo".to_string(), vt_1);
    db.add("despido".to_string(), vt_2);
    db.add("saludo_despido".to_string(), vt_3);
    db.add("valores".to_string(), vt_4);

    let aux = db.get_string_value_by_key("saludo").unwrap();
    assert_eq!(aux, String::from("hola"));
    let aux = db.get_string_value_by_key("despido").unwrap();
    assert_eq!(aux, String::from("chau"));
    let aux = db.get_string_value_by_key("saludo_despido");
    assert!(aux.is_none());
    let aux = db.get_string_value_by_key("valores");
    assert!(aux.is_none());
    let _ = std::fs::remove_file("file025".to_string());
}

#[test]
fn test_28_scard_de_set_existente_devuelve_cantidad_de_elementos() {
    use std::collections::HashSet;
    let mut db = Database::new("file026".to_string());
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());
    this_set.insert("value_2".to_string());

    let vt = ValueTimeItem::new_now(ValueType::SetType(this_set), KeyAccessTime::Persistent);

    db.items.insert("valores".to_string(), vt);
    let len = db.get_len_of_set("valores");
    assert_eq!(len, 2);
    let _ = std::fs::remove_file("file026".to_string());
}

#[test]
fn test_29_scard_de_set_devuelve_cero_si_no_existe() {
    let mut db = Database::new("file027".to_string());

    let len = db.get_len_of_set("valores");
    assert_eq!(len, 0);
    let _ = std::fs::remove_file("file027".to_string());
}

#[test]
fn test_30_scard_de_set_devuelve_cero_si_no_es_tipo_set() {
    let vt_1 = ValueTimeItem::new_now(
        ValueType::StringType("hola".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let mut db = Database::new("file028".to_string());
    db.items.insert("saludo".to_string(), vt_1);

    let len = db.get_len_of_set("saludo");
    assert_eq!(len, 0);
    let _ = std::fs::remove_file("file028".to_string());
}

#[test]
fn test_31_ismember_de_set_devuelve_cero_si_no_es_tipo_set() {
    let vt_1 = ValueTimeItem::new_now(
        ValueType::StringType("hola".to_string()),
        KeyAccessTime::Volatile(0),
    );
    let mut db = Database::new("file029".to_string());
    db.items.insert("saludo".to_string(), vt_1);

    let len = db.is_member_of_set("saludo", "hola");
    assert_eq!(len, 0);
    let _ = std::fs::remove_file("file029".to_string());
}

#[test]
fn test_31_ismember_de_set_devuelve_cero_si_no_existe_clave() {
    let mut db = Database::new("file030".to_string());

    let len = db.is_member_of_set("valores", "hola");
    assert_eq!(len, 0);
    let _ = std::fs::remove_file("file030".to_string());
}

#[test]
fn test_32_ismember_de_set_existente_devuelve_uno() {
    use std::collections::HashSet;
    let mut db = Database::new("file031".to_string());
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());

    let vt = ValueTimeItem::new_now(ValueType::SetType(this_set), KeyAccessTime::Persistent);

    db.items.insert("valores".to_string(), vt);
    let is_member = db.is_member_of_set("valores", "value_1");
    assert_eq!(is_member, 1);
    let _ = std::fs::remove_file("file031".to_string());
}

#[test]
fn test_33_ismember_de_set_existente_devuelve_cero_si_no_pertenece_al_set() {
    use std::collections::HashSet;
    let mut db = Database::new("file032".to_string());
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());

    let vt = ValueTimeItem::new_now(ValueType::SetType(this_set), KeyAccessTime::Persistent);

    db.items.insert("valores".to_string(), vt);
    let is_member = db.is_member_of_set("valores", "value_2");
    assert_eq!(is_member, 0);
    let _ = std::fs::remove_file("file032".to_string());
}

#[test]
fn test_34_get_members_of_set_existente_devuelve_elementos_del_set() {
    use std::collections::HashSet;
    let mut db = Database::new("file033".to_string());
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());
    this_set.insert("value_2".to_string());
    this_set.insert("value_3".to_string());

    let vt = ValueTimeItem::new_now(ValueType::SetType(this_set), KeyAccessTime::Persistent);

    db.items.insert("valores".to_string(), vt);
    let members = db.get_members_of_set("valores");
    assert!(members.contains(&&String::from("value_1")));
    assert!(members.contains(&&String::from("value_2")));
    assert!(members.contains(&&String::from("value_3")));
    assert_eq!(members.len(), 3);
    let _ = std::fs::remove_file("file033".to_string());
}

#[test]
fn test_35_remove_member_from_existing_set_returns_true() {
    use std::collections::HashSet;
    let mut db = Database::new("file034".to_string());
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());
    this_set.insert("value_2".to_string());

    let vt = ValueTimeItem::new_now(ValueType::SetType(this_set), KeyAccessTime::Persistent);

    db.items.insert("valores".to_string(), vt);
    let removed = db.remove_member_from_set("valores", "value_1").unwrap();
    assert_eq!(removed, true);

    let _ = std::fs::remove_file("file034".to_string());
}

#[test]
fn test_36_remove_member_from_non_existing_set_returns_false() {
    use std::collections::HashSet;
    let mut db = Database::new("file035".to_string());
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());
    this_set.insert("value_2".to_string());

    let vt = ValueTimeItem::new_now(ValueType::SetType(this_set), KeyAccessTime::Persistent);

    db.items.insert("values".to_string(), vt);
    let removed = db.remove_member_from_set("valores", "value_1").unwrap();
    assert_eq!(removed, false);

    let _ = std::fs::remove_file("file035".to_string());
}

#[test]
fn test_37_remove_member_from_list_type_returns_none() {
    let mut db = Database::new("file036".to_string());
    let vt = ValueTimeItem::new_now(
        ValueType::ListType(vec!["hola".to_string(), "chau".to_string()]),
        KeyAccessTime::Persistent,
    );

    db.items.insert("saludo".to_string(), vt);
    let removed = db.remove_member_from_set("saludo", "value_1");
    assert!(removed.is_none());

    let _ = std::fs::remove_file("file036".to_string());
}

#[test]
fn test_38_pop_one_element_from_list_returns_popped_element() {
    let mut db = Database::new("file037".to_string());
    let vt = ValueTimeItem::new_now(
        ValueType::ListType(vec!["hola".to_string(), "chau".to_string()]),
        KeyAccessTime::Persistent,
    );

    db.items.insert("saludo".to_string(), vt);
    let removed = db.pop_elements_from_list("saludo", 1).unwrap();
    assert_eq!(removed, vec![String::from("hola")]);
    let item = db.get_live_item("saludo").unwrap();
    if let ValueType::ListType(item) = item.get_value() {
        assert_eq!(item, &vec![String::from("chau")]);
    } else {
        assert!(false);
    }

    let _ = std::fs::remove_file("file037".to_string());
}

#[test]
fn test_39_pop_multiple_elements_from_list_returns_popped_elements() {
    let mut db = Database::new("file038".to_string());
    let vt = ValueTimeItem::new_now(
        ValueType::ListType(vec![
            "hola".to_string(),
            "chau".to_string(),
            "hello".to_string(),
            "bye".to_string(),
        ]),
        KeyAccessTime::Persistent,
    );

    db.items.insert("saludo".to_string(), vt);
    let removed = db.pop_elements_from_list("saludo", 2).unwrap();
    assert_eq!(removed, vec![String::from("hola"), String::from("chau")]);
    let item = db.get_live_item("saludo").unwrap();
    if let ValueType::ListType(item) = item.get_value() {
        assert!(item.contains(&String::from("hello")) && item.contains(&String::from("bye")));
    } else {
        assert!(false);
    }

    let _ = std::fs::remove_file("file038".to_string());
}
