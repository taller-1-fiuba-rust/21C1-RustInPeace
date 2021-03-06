//! Base de datos.

use crate::domain::entities::key_value_item::{KeyAccessTime, ValueTimeItem};
use crate::domain::entities::key_value_item::{ValueTimeItemBuilder, ValueType};
use crate::domain::entities::key_value_item_serialized::KeyValueItemSerialized;
use crate::errors::database_error::DatabaseError;
use crate::services::utils::glob_pattern;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::Write;
use std::io::{self};
use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;

/// Estructura que almacena todos los datos. Está compuesta por la dirección del archivo donde periodicamente se bajan los datos y por un HashMap que contiene
/// todos los datos en la forma (clave, valor), siendo la clave de tipo `String` y el valor de tipo `ValueTimeItem`.
#[derive(Debug)]
pub struct Database {
    dbfilename: String,
    items: HashMap<String, ValueTimeItem>,
}

impl Database {
    /// Crea una nueva instancia de Database.
    ///
    /// Añade todos los datos almacenados en el archivo `filename`.
    ///
    /// # Ejemplo
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// Database::new("dummy_db.csv".to_string());
    /// # let _ = std::fs::remove_file("dummy_db.csv");
    /// ```
    pub fn new(filename: String) -> Database {
        let mut db = Database {
            dbfilename: filename,
            items: HashMap::new(),
        };
        db.load_items();
        db
    }

    /// Retorna la dirección del archivo database.
    ///
    /// # Ejemplo
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// let db = Database::new("dummy_db.csv".to_string());
    /// assert_eq!(db.get_filename(), &"dummy_db.csv".to_string());
    /// # let _ = std::fs::remove_file("dummy_db.csv");
    /// ```
    pub fn get_filename(&self) -> &String {
        &self.dbfilename
    }

    // pub fn get_items(&self) -> &HashMap<String, ValueTimeItem> {
    //     &self.items
    // }

    /// Retorna los datos almacenados.
    ///
    /// Chequea si la clave está expirada, si lo está la elimina.
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// # let mut db = Database::new("dummy_db_items.csv".to_string());
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("perro".to_string())).build();
    /// db.add("tipo_mascota".to_string(), vt);
    ///
    /// match db.get_live_item("tipo_mascota").unwrap().get_value() {
    ///     ValueType::StringType(tipo) => {assert_eq!(tipo, &String::from("perro"))}
    ///     _ => assert!(false)
    /// }
    /// # let _ = std::fs::remove_file("dummy_db_items.csv");
    /// ```
    pub fn get_live_item(&mut self, key: &str) -> Option<&ValueTimeItem> {
        let (items, expired) = self.check_timeout_item(key);
        if items.is_some() && expired {
            self.items.remove(key).take();
        }
        self.items.get(key)
    }

    /// Retorna los datos almacenados.
    ///
    /// Chequea si la clave está expirada, si lo está la elimina. A diferencia de `get_live_item`, esta función retorna los items mutables.
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// # let mut db = Database::new("dummy_db_items.csv".to_string());
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("perro".to_string())).build();
    /// db.add("tipo_mascota".to_string(), vt);
    ///
    /// match db.get_mut_live_item("tipo_mascota").unwrap().get_value() {
    ///     ValueType::StringType(tipo) => {assert_eq!(tipo, &String::from("perro"))}
    ///     _ => assert!(false)
    /// }
    /// # let _ = std::fs::remove_file("dummy_db_items.csv");
    /// ```
    pub fn get_mut_live_item(&mut self, key: &str) -> Option<&mut ValueTimeItem> {
        let (items, expired) = self.check_timeout_item(key);
        if items.is_some() && expired {
            self.items.remove(key);
        }
        self.items.get_mut(key)
    }

    /// Chequea si la clave expiró.
    ///
    /// Devuelve una tupla con el item como Optional y un booleano indicando si está expirada
    /// Si está expirada: (Some,true) sino devuelve el valor almacenado en dicha clave como (Some,false)
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// # let mut db = Database::new("dummy_db_check_timeout.csv".to_string());
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("perro".to_string())).build();
    /// db.add("tipo_mascota".to_string(), vt);
    ///
    /// assert!(db.check_timeout_item("tipo_mascota").0.is_some());
    /// assert_eq!(db.check_timeout_item("tipo_mascota").1,false);
    /// # let _ = std::fs::remove_file("dummy_db_check_timeout.csv");
    /// ```
    pub fn check_timeout_item(&self, key: &str) -> (Option<&ValueTimeItem>, bool) {
        let option_item = self.items.get(key);
        if let Some(item) = option_item {
            let expired = item.is_expired();
            return (Some(item), expired);
        }
        (None, false)
    }

    /// Elimina todos las claves (y sus valores asociados) de la base de datos.
    ///
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// # let mut db = Database::new("dummy_db_clean.csv".to_string());
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("perro".to_string())).build();
    /// db.add("tipo_mascota".to_string(), vt);
    /// db.clean_items();
    /// assert!(db.get_live_item("tipo_mascota").is_none());
    /// # let _ = std::fs::remove_file("dummy_db_clean.csv");
    /// ```
    pub fn clean_items(&mut self) -> &HashMap<String, ValueTimeItem> {
        self.items.clear();
        &self.items
    }

    /// Devuelve todas las claves que coinciden con el patrón.
    ///
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// # let mut db = Database::new("dummy_db_matching_keys.csv".to_string());
    /// let mut list = vec![String::from("pedro"), String::from("luis"), String::from("juan"), String::from("pepe")];
    /// let vt = ValueTimeItemBuilder::new(ValueType::ListType(list)).build();
    /// db.add("amigos".to_string(), vt);
    ///
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("25".to_string())).build();
    /// db.add("edad_pedro".to_string(), vt);
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("23".to_string())).build();
    /// db.add("edad_luis".to_string(), vt);
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("35".to_string())).build();
    /// db.add("edad_juan".to_string(), vt);
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("22".to_string())).build();
    /// db.add("ed_pepe".to_string(), vt);
    ///
    /// let matching = db.get_keys_that_match_pattern("edad*");
    /// assert!(matching.contains(&"edad_pedro".to_string()));
    /// assert!(matching.contains(&"edad_luis".to_string()));
    /// assert!(matching.contains(&"edad_juan".to_string()));
    /// assert!(!matching.contains(&"ed_pepe".to_string()));
    ///
    /// # let _ = std::fs::remove_file("dummy_db_matching_keys.csv");
    /// ```
    pub fn get_keys_that_match_pattern(&self, pattern: &str) -> Vec<String> {
        let mut matching_keys = vec![];
        for key in &self.items {
            let current_key = key.0.to_string();
            if !key.1.is_expired()
                && glob_pattern::g_match(pattern.as_bytes(), current_key.as_bytes())
            {
                matching_keys.push(current_key);
            }
        }
        matching_keys
    }

    /// Devuelve si la clave existe en la base de datos.
    ///
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// # let mut db = Database::new("dummy_db_exists.csv".to_string());
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("perro".to_string())).build();
    /// db.add("tipo_mascota".to_string(), vt);
    /// assert!(db.key_exists("tipo_mascota".to_string()));
    /// assert!(!db.key_exists("nombre_mascota".to_string()));
    /// # let _ = std::fs::remove_file("dummy_db_exists.csv");
    /// ```
    pub fn key_exists(&mut self, key: String) -> bool {
        return self.get_live_item(&key).is_some();
    }

    /// Devuelve si la clave existe en la base de datos y si está expirada.
    ///
    /// Retorna una tupla con true/false dependiendo si la clave existe, y en caso de que exista el estado de la misma.
    /// Expired también en formato booleano.
    ///
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// # let mut db = Database::new("dummy_db_exists_expired.csv".to_string());
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("perro".to_string())).with_timeout(123).build();
    /// db.add("tipo_mascota".to_string(), vt);
    ///
    /// assert_eq!(db.key_exists_expired("tipo_mascota".to_string()).1,true);
    /// assert!(!db.key_exists_expired("nombre_mascota".to_string()).0);
    /// # let _ = std::fs::remove_file("dummy_db_exists_expired.csv");
    /// ```
    pub fn key_exists_expired(&self, key: String) -> (bool, bool) {
        let (items, expired) = self.check_timeout_item(&key);
        (items.is_some(), expired)
    }

    /// Elimina una clave de la base de datos
    ///
    /// Permite que el usuario elimine una clave expirada. No devuelve nada.
    ///
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// # let mut db = Database::new("dummy_db_key_expired.csv".to_string());
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("perro".to_string())).with_timeout(123).build();
    /// db.add("tipo_mascota".to_string(), vt);
    ///
    /// db.remove_expired_key("tipo_mascota");
    /// assert!(!db.key_exists_expired("tipo_mascota".to_string()).0);
    /// # let _ = std::fs::remove_file("dummy_db_key_expired.csv");
    /// ```
    pub fn remove_expired_key(&mut self, key: &str) {
        self.items.remove(key);
    }

    /// Agrega a la base de datos una `key` con un `value` asociado.
    ///
    /// Si la clave ya existe, actualiza su valor.
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// # let mut db = Database::new("dummy_db_add.csv".to_string());
    /// let mut list = vec![String::from("pedro"), String::from("luis"), String::from("juan"), String::from("pepe")];
    /// let vt = ValueTimeItemBuilder::new(ValueType::ListType(list)).build();
    /// db.add("amigos".to_string(), vt);
    ///
    /// match db.get_live_item("amigos").unwrap().get_value() {
    ///     ValueType::ListType(list) => {assert_eq!(list, &vec![String::from("pedro"), String::from("luis"), String::from("juan"), String::from("pepe")])}
    ///     _ => assert!(false)
    /// }
    /// # let _ = std::fs::remove_file("dummy_db_add.csv");
    /// ```
    pub fn add(&mut self, key: String, value: ValueTimeItem) {
        self.items.insert(key, value);
    }

    /// Busca los valores de las claves asociadas al patrón especificado.
    ///
    /// Reemplaza el "*" del patrón por cada elemento perteneciente a `key` y busca su valor asociado.
    /// Por ejemplo, si se tiene la clave "Amigos" con los valores "Pedro", "Luis", "Juan" y para cada uno de estos valores, hay una clave asociada del tipo "edad_{Nombre}".
    /// Si el patrón es "edad_*", la función devuelve: [("Pedro", 25), ("Luis", 23), ("Juan", 35)]
    ///
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// # let mut db = Database::new("dummy_db_external_keys.csv".to_string());
    /// let mut list = vec![String::from("pedro"), String::from("luis"), String::from("juan"), String::from("pepe")];
    /// let vt = ValueTimeItemBuilder::new(ValueType::ListType(list)).build();
    /// db.add("amigos".to_string(), vt);
    ///
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("25".to_string())).build();
    /// db.add("edad_pedro".to_string(), vt);
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("23".to_string())).build();
    /// db.add("edad_luis".to_string(), vt);
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("35".to_string())).build();
    /// db.add("edad_juan".to_string(), vt);
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("22".to_string())).build();
    /// db.add("edad_pepe".to_string(), vt);
    ///
    /// let (values,expired) = db.get_values_of_keys_matching_pattern("edad_*".to_string(), "amigos".to_string());
    /// assert!(values.contains(&("pedro".to_string(), "25".to_string())));
    /// assert!(values.contains(&("luis".to_string(), "23".to_string())));
    /// assert!(values.contains(&("juan".to_string(), "35".to_string())));
    /// assert!(values.contains(&("pepe".to_string(), "22".to_string())));
    ///
    /// # let _ = std::fs::remove_file("dummy_db_external_keys.csv");
    /// ```
    pub fn get_values_of_keys_matching_pattern(
        &self,
        pat: String,
        key: String,
    ) -> (Vec<(String, String)>, Vec<String>) {
        let mut associated_values = Vec::new();
        let mut expired_items = Vec::new();
        if let (Some(item), expired) = self.check_timeout_item(&key) {
            if expired {
                expired_items.push(key);
            }
            let elements: Vec<String> = item
                .get_value_as_vec()
                .iter()
                .map(|e| e.to_string())
                .collect();
            elements.iter().for_each(|element| {
                let patterned_key = pat.replace('*', element.as_str());
                if self.items.contains_key(&patterned_key) {
                    if let ValueType::StringType(value) =
                        self.items.get(&patterned_key).unwrap().get_value()
                    {
                        associated_values.push((element.to_string(), value.to_string()));
                    }
                }
            });
        }
        (associated_values, expired_items)
    }

    /// Agrega nuevos elementos a una lista.
    ///
    /// A diferencia de `push_vec_to_list`, esta función agrega los nuevos elementos al comienzo de la lista.
    /// Si `strict` es True, sólo agrega elementos a una clave ya existente, si la clave no existe no hace nada.
    /// Si `strict` es False y la clave no existe, la crea.
    /// Devuelve la cantidad de elementos de la lista luego de haber agregaro los nuevos valores.
    ///
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// # let mut db = Database::new("dummy_db_addtolist.csv".to_string());
    /// let mut list = vec![String::from("pedro"), String::from("luis"), String::from("juan"), String::from("pepe")];
    /// let vt = ValueTimeItemBuilder::new(ValueType::ListType(list)).build();
    /// db.add("amigos".to_string(), vt);
    ///
    /// let added = db.add_to_list_type(vec!["pepe".to_string(), "bruno".to_string()], "amigos", true).unwrap();
    /// assert_eq!(added, 6);
    ///
    /// # let _ = std::fs::remove_file("dummy_db_addtolist.csv");
    /// ```
    pub fn add_to_list_type(
        &mut self,
        mut new_vec: Vec<String>,
        key: &str,
        strict: bool,
    ) -> Option<usize> {
        if self.key_exists(key.to_string()) {
            if let ValueType::ListType(current_value) =
                self.get_live_item(key).unwrap().get_value().to_owned()
            {
                let mut old_vector = current_value;
                new_vec.append(&mut old_vector);
                let len = new_vec.len();
                let vt_item = ValueTimeItemBuilder::new(ValueType::ListType(new_vec)).build();
                self.add(key.to_string(), vt_item);
                return Some(len);
            }
        } else if !strict {
            let vec_len = new_vec.len();
            let vt_item = ValueTimeItemBuilder::new(ValueType::ListType(new_vec)).build();
            self.add(key.to_string(), vt_item);
            return Some(vec_len);
        }
        None
    }

    /// Inserta nuevos elementos al final de la lista almacenada en `key`.
    ///
    /// Si la clave existe y guarda un elemento de tipo lista, inserta los elementos al final de la misma.
    /// Retorna la longitud de la lista luego de haber insertado los nuevos elementos.
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_rpush.csv".to_string());
    /// let mut list = vec![String::from("argentina"), String::from("brasil"), String::from("chile"), String::from("uruguay")];
    /// let vt = ValueTimeItemBuilder::new(ValueType::ListType(list)).build();
    /// db.add("paises".to_string(), vt);
    ///
    /// let len = db.push_vec_to_list(vec![String::from("bolivia"), String::from("paraguay")], "paises");
    /// assert_eq!(len, 6);
    ///
    /// # std::fs::remove_file("dummy_db_rpush.csv");
    /// ```
    pub fn push_vec_to_list(&mut self, new_elements: Vec<String>, key: &str) -> usize {
        if let Some(item) = self.get_mut_live_item(key) {
            if let ValueType::ListType(mut current_value) = item.get_copy_of_value() {
                for element in new_elements {
                    current_value.push(element);
                }
                let len = current_value.len();
                item.set_value(ValueType::ListType(current_value));
                return len;
            }
        }
        0
    }

    /// Reemplaza el elemento en la posición `index` de la lista.
    ///
    /// Si el indice es negativo, le suma el valor de la longitud de la lista.
    /// Devuelve True en caso de éxito, sino False.
    ///
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_replace.csv".to_string());
    /// let mut list = vec![String::from("argentina"), String::from("brasil"), String::from("chile")];
    /// let vt = ValueTimeItemBuilder::new(ValueType::ListType(list)).build();
    /// db.add("paises".to_string(), vt);
    ///
    /// let replaced = db.replace_element_in_list_type_value("paises", "uruguay", "1");
    /// assert!(replaced);
    ///
    /// # std::fs::remove_file("dummy_db_replace.csv");
    /// ```
    pub fn replace_element_in_list_type_value(
        &mut self,
        key: &str,
        value: &str,
        index: &str,
    ) -> bool {
        if let Some(item) = self.get_mut_live_item(key) {
            if let ValueType::ListType(mut current_value) = item.get_copy_of_value() {
                let current_value_len = current_value.len() as isize;
                let mut current_index = index.parse::<isize>().unwrap_or(current_value_len + 1);
                if current_index < 0 {
                    current_index += current_value_len;
                };
                if current_index < current_value_len {
                    current_value[current_index as usize] = value.to_string();
                    item.set_value(ValueType::ListType(current_value));
                    return true;
                }
            }
        }
        false
    }

    /// Devuelve una porcion de una lista asociada a una key que almacena un valor de tipo List.
    ///
    /// Busca los elementos cuya posición se encuentra en el rango [`lower_bound`, `upper_bound`].
    /// Si alguno de los extremos es negativo, le suma la longitud de la lista original.
    ///
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_range.csv".to_string());
    /// let mut list = vec![String::from("argentina"), String::from("brasil"), String::from("chile")];
    /// let vt = ValueTimeItemBuilder::new(ValueType::ListType(list)).build();
    /// db.add("paises".to_string(), vt);
    ///
    /// let values = db.get_values_in_range("paises", "1", "2").unwrap();
    /// assert_eq!(values, vec!["brasil".to_string(), "chile".to_string()]);
    ///
    /// # std::fs::remove_file("dummy_db_range.csv");
    /// ```
    pub fn get_values_in_range(
        &self,
        key: &str,
        lower_bound: &str,
        upper_bound: &str,
    ) -> Option<Vec<String>> {
        if let (Some(item), false) = self.check_timeout_item(key) {
            if let ValueType::ListType(current_value) = item.get_value().to_owned() {
                let current_value_len = current_value.len() as isize;
                let mut vec_values_selected_by_index = vec![];
                let mut lb = lower_bound.parse::<isize>().unwrap_or(0);
                let mut ub = upper_bound.parse::<isize>().unwrap_or(current_value_len);
                if lb < 0 {
                    lb += current_value_len;
                }
                if ub < 0 {
                    ub += current_value_len;
                }
                if ub > lb {
                    if ub <= current_value_len {
                        for j in lb..(ub + 1) {
                            vec_values_selected_by_index.push(current_value[j as usize].to_owned());
                        }
                    } else {
                        for j in lb..current_value_len {
                            vec_values_selected_by_index.push(current_value[j as usize].to_owned());
                        }
                    }
                    Some(vec_values_selected_by_index)
                } else {
                    Some(vec!["".to_string()])
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Devuelve el valor de la lista en la posición `index`.
    ///
    /// Devuelve None si el valor almacenado en `key` no es de tipo lista.
    /// Si el indice es menor a 0, le suma la longitud original de la lista.
    ///
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_index.csv".to_string());
    /// let mut list = vec![String::from("argentina"), String::from("brasil"), String::from("chile")];
    /// let vt = ValueTimeItemBuilder::new(ValueType::ListType(list)).build();
    /// db.add("paises".to_string(), vt);
    ///
    /// let value = db.get_value_by_index("paises", "1").unwrap();
    /// assert_eq!(value, "brasil".to_string());
    ///
    /// # std::fs::remove_file("dummy_db_index.csv");
    /// ```
    pub fn get_value_by_index(&self, key: &str, index: &str) -> Result<String, DatabaseError> {
        if let (Some(item), false) = self.check_timeout_item(key) {
            let mut index_aux = index.parse::<isize>().unwrap();
            if let ValueType::ListType(current_value) = item.get_value().to_owned() {
                let current_value_len = current_value.len() as isize;
                if index_aux >= current_value_len {
                    return Err(DatabaseError::InvalidValueType(
                        "index out of bounds".to_string(),
                    ));
                }
                if index_aux < 0 {
                    index_aux += current_value_len;
                };
                Ok(current_value[index_aux as usize].to_owned())
            } else {
                Err(DatabaseError::InvalidValueType("Not list type".to_string()))
            }
        } else {
            Ok("".to_string())
        }
    }

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
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    /// use std::thread::sleep;
    /// use std::time::Duration;
    ///
    /// // Agrego los datos en la base de datos
    /// let mut db = Database::new("dummy_db_doc_reboot.csv".to_string());
    /// db.add("altura_juan".to_string(),ValueTimeItemBuilder::new(
    /// ValueType::StringType("1.78".to_string())
    /// ).build());
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
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueType, ValueTimeItem, KeyAccessTime, ValueTimeItemBuilder};
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
    /// db.add("altura_juan".to_string(),ValueTimeItemBuilder::new(
    /// ValueType::StringType("1.78".to_string())).with_timeout(timeout).build());
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
    /// ```
    pub fn reboot_time(&mut self, key: String) -> Option<&mut ValueTimeItem> {
        let mut item = self.get_mut_live_item(&key);
        if let Some(item) = &mut item {
            item.reboot_last_access_time();
        }
        item
    }

    /// Devuelve el tipo de dato del value.
    ///
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_valuetype.csv".to_string());
    /// let mut list = vec![String::from("argentina"), String::from("brasil"), String::from("chile")];
    /// let vt = ValueTimeItemBuilder::new(ValueType::ListType(list)).build();
    /// db.add("paises".to_string(), vt);
    ///
    /// let value_type = db.get_type_of_value("paises".to_string());
    /// assert_eq!(value_type, "list".to_string());
    ///
    /// # std::fs::remove_file("dummy_db_valuetype.csv");
    /// ```
    pub fn get_type_of_value(&self, key: String) -> String {
        if let Some(value) = self.items.get(&key) {
            value.get_value_type()
        } else {
            "none".to_string()
        }
    }

    /// Copia el valor almacenado en una clave origen a una clave destino.
    ///
    /// Si el parámetro `replace` es true, entonces reemplaza el valor almacenado en la clave destino
    /// por el valor de la clave origen. Si es false y la clave destino ya existe, devuelve None.
    /// Si la clave destino no existe, se crea.
    /// # Example
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, KeyAccessTime, ValueType, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_copy.csv".to_string());
    /// db.add("dolly".to_string(),ValueTimeItemBuilder::new(
    /// ValueType::StringType("sheep".to_string())).build()
    /// );
    /// db.copy(String::from("dolly"), String::from("clone"), true);
    ///
    /// let copied = db.get_live_item("clone").unwrap();
    /// if let ValueType::StringType(str) = copied.get_value() {
    ///    assert_eq!(str, &String::from("sheep"));
    /// }
    ///
    /// # std::fs::remove_file("dummy_db_copy.csv");
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
                    self.add(
                        destination,
                        ValueTimeItemBuilder::new(new_value)
                            .with_key_access_time(timeout)
                            .build(),
                    );
                    Some(())
                }
            }
        } else {
            None
        };
    }

    /// Persiste una clave.
    ///
    /// Si la clave existe, cambia el tipo de clave a Persistente, es decir que no expira y devuelve True.
    /// Si la clave no existe, devuelve False.
    ///
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, KeyAccessTime, ValueType, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_persist.csv".to_string());
    /// db.add("dolly".to_string(), ValueTimeItemBuilder::new(
    ///     ValueType::StringType("sheep".to_string())).with_timeout(1825601548).build()
    /// );
    ///
    /// assert!(db.persist("dolly".to_string()));
    ///
    /// # std::fs::remove_file("dummy_db_persist.csv");
    /// ```
    pub fn persist(&mut self, key: String) -> bool {
        match self.get_mut_live_item(&key) {
            Some(item) => item.make_persistent(),
            None => false,
        }
    }

    /// Renombra una clave.
    ///
    /// Si la clave existe, se renombra y sobreescribe con una copia del valor almacenado.
    /// En ese caso, el método devuelve True, sino False.
    ///
    /// # Ejemplo
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// // Agrego los datos en la base de datos
    /// let mut db = Database::new("dummy_db_rename.csv".to_string());
    /// db.add("dolly".to_string(),ValueTimeItemBuilder::new(
    /// ValueType::StringType("sheep".to_string())
    /// ).build());
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
            self.delete_key(current_key);
            self.add(
                new_key,
                ValueTimeItemBuilder::new(item_value)
                    .with_key_access_time(item_time)
                    .build(),
            );
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
    /// # Ejemplo
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// // Agrego los datos en la base de datos
    /// let mut db = Database::new("dummy_db_append.csv".to_string());
    /// db.add("field".to_string(),
    /// ValueTimeItemBuilder::new(ValueType::StringType("first".to_string())).build());
    ///
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
                    ValueTimeItemBuilder::new(ValueType::StringType(string.to_string())).build(),
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
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// // Agrego los datos en la base de datos
    /// let mut db = Database::new("dummy_db_decrement.csv".to_string());
    /// db.add("edad".to_string(),ValueTimeItemBuilder::new(
    /// ValueType::StringType("25".to_string())
    /// ).build());
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
                            "Value at key can't be represented as integer",
                        )))
                    }
                } else {
                    Err(DatabaseError::InvalidValueType(format!(
                        "Invalid value type. Expected: string. Got: {}",
                        item.get_value_type()
                    )))
                }
            }
            None => {
                let new_value = 0 - decr;
                self.items.insert(
                    key.to_string(),
                    ValueTimeItemBuilder::new(ValueType::StringType(new_value.to_string())).build(),
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
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// // Agrego los datos en la base de datos
    /// let mut db = Database::new("dummy_db_increment.csv".to_string());
    /// db.add("edad".to_string(),
    /// ValueTimeItemBuilder::new(ValueType::StringType("25".to_string())).build()
    /// );
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
                        "Value at key can't be represented as integer",
                    )))
                }
            } else {
                return Err(DatabaseError::InvalidValueType(format!(
                    "Invalid value type. Expected: string. Got: {}",
                    item.get_value_type()
                )));
            }
        } else {
            let new_value = incr;
            self.items.insert(
                key.to_string(),
                ValueTimeItemBuilder::new(ValueType::StringType(new_value.to_string())).build(),
            );
            Ok(new_value)
        }
    }

    /// Devuelve el valor almacenado en `key` si dicho valor es de tipo String.
    ///
    /// Retorna Error si la clave no existe o si el valor almacenado no es de tipo String.
    ///
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_get_string".to_string());
    /// let vt_1 = ValueTimeItemBuilder::new(
    ///    ValueType::StringType("hola".to_string())
    /// ).build();
    /// let vt_2 = ValueTimeItemBuilder::new(
    ///    ValueType::ListType(vec!["hola".to_string(), "chau".to_string()])
    /// ).build();
    /// db.add("saludo".to_string(), vt_1);
    /// db.add("saludo_despido".to_string(), vt_2);
    /// let aux = db.get_string_value_by_key("saludo").unwrap();
    /// assert_eq!(aux, String::from("hola"));
    /// let aux = db.get_string_value_by_key("saludo_despido");
    /// assert!(aux.is_err());
    ///
    /// let _ = std::fs::remove_file("dummy_db_get_string");
    /// ```
    pub fn get_string_value_by_key(&self, key: &str) -> Result<String, DatabaseError> {
        if let (Some(item), false) = self.check_timeout_item(&key.to_string()) {
            let value = item.get_copy_of_value();
            if let ValueType::StringType(str) = value {
                return Ok(str);
            } else {
                return Err(DatabaseError::InvalidValueType(
                    "Expected string, got another value type.".to_string(),
                ));
            }
        }
        Err(DatabaseError::MissingKey())
    }

    /// Retorna la longitud del string almacenado en `key`.
    ///
    /// Si la clave existe y el valor almacenado es de tipo String, retorna la longitud del mismo.
    /// Si la clave no existe, retorna 0. Retorna None si el valor almacenado no es de tipo String.
    ///
    /// # Example
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_strlen.csv".to_string());
    /// db.add("mykey".to_string(),ValueTimeItemBuilder::new(
    /// ValueType::StringType("myvalue".to_string())
    /// ).build());
    /// let len = db.get_strlen_by_key("mykey").unwrap();
    ///
    /// assert_eq!(len, 7);
    ///
    /// let _ = std::fs::remove_file("dummy_db_strlen.csv");
    /// ```
    pub fn get_strlen_by_key(&self, key: &str) -> Option<usize> {
        if let (Some(item), false) = self.check_timeout_item(key) {
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
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_getdel.csv".to_string());
    /// db.add("mykey".to_string(),ValueTimeItemBuilder::new(
    /// ValueType::StringType("myvalue".to_string())
    /// ).build());
    /// let deleted_value = db.getdel_value_by_key("mykey").unwrap();
    ///
    /// assert_eq!(deleted_value, String::from("myvalue"));
    ///
    /// let _ = std::fs::remove_file("dummy_db_getdel.csv");
    /// ```
    pub fn getdel_value_by_key(&mut self, key: &str) -> Result<String, DatabaseError> {
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
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_getset.csv".to_string());
    /// db.add("mykey".to_string(),ValueTimeItemBuilder::new(
    /// ValueType::StringType("myvalue".to_string())
    /// ).build());
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
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    /// use std::collections::HashSet;
    ///
    /// let mut db = Database::new("dummy_db_setlen.csv".to_string());
    /// let mut set = HashSet::new();
    /// set.insert("25".to_string());
    /// set.insert("40".to_string());
    /// let vt = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    /// db.add("edades".to_string(), vt);
    ///
    /// let len = db.get_len_of_set("edades");
    ///
    /// assert_eq!(len, 2);
    ///
    /// let _ = std::fs::remove_file("dummy_db_setlen.csv");
    /// ```
    pub fn get_len_of_set(&self, key: &str) -> usize {
        let (item, expire) = self.check_timeout_item(key);
        if expire {
            return 0;
        }
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

    /// Retorna si el elemento pertenece al set almacenado en `key`.
    ///
    /// Si la clave no existe o el valor no es de tipo Set, devuelve 0.
    /// Si el elemento pertenece al set, devuelve 1.
    /// # Example
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    /// # use std::collections::HashSet;
    ///
    /// let mut db = Database::new("dummy_db_ismember.csv".to_string());
    /// let mut set = HashSet::new();
    /// set.insert("25".to_string());
    /// set.insert("40".to_string());
    /// let vt = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    /// db.add("edades".to_string(), vt);
    ///
    /// assert_eq!(1, db.is_member_of_set("edades", "40"));
    /// assert_eq!(0, db.is_member_of_set("edades", "45"));
    ///
    /// # let _ = std::fs::remove_file("dummy_db_ismember.csv");
    /// ```
    pub fn is_member_of_set(&self, key: &str, member: &str) -> usize {
        if let (Some(item), false) = self.check_timeout_item(key) {
            if let ValueType::SetType(item) = item.get_value() {
                if item.get(member).is_some() {
                    return 1;
                }
            }
        }
        0
    }

    /// Retorna los elementos que pertenecen al set almacenado en `key`.
    ///
    /// Valida que el valor almacenado en `key` sea de tipo Set, sino devuelve un vector vacío.
    /// Devuelve un vector con todos los elementos del set.
    /// # Example
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    /// # use std::collections::HashSet;
    ///
    /// let mut db = Database::new("dummy_db_getmembers.csv".to_string());
    /// let mut set = HashSet::new();
    /// set.insert("25".to_string());
    /// set.insert("40".to_string());
    /// let vt = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    /// db.add("edades".to_string(), vt);
    ///
    /// let res = db.get_members_of_set("edades");
    /// assert!(res.contains(&&"40".to_string()));
    /// assert!(res.contains(&&"25".to_string()));
    ///
    /// # let _ = std::fs::remove_file("dummy_db_getmembers.csv");
    /// ```
    pub fn get_members_of_set(&self, key: &str) -> Vec<&String> {
        let (item, expired) = self.check_timeout_item(key);
        let mut members = Vec::new();
        if let Some(item) = item {
            if let ValueType::SetType(item) = item.get_value() {
                item.iter().for_each(|member| {
                    if !expired {
                        members.push(member);
                    }
                });
            }
        }
        members
    }

    /// Elimina un elemento del set almacenado en `key`.
    ///
    /// Si la clave no existe o el set está vacío, devuelve false.
    /// Si el valor almacenado en `key` no es de tipo Set o el elemento no pertecene al set, devuelve None.
    /// Si se eliminan todos los elementos pedidos, devuelve True.
    /// # Example
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    /// # use std::collections::HashSet;
    ///
    /// let mut db = Database::new("dummy_db_rmember.csv".to_string());
    /// let mut set = HashSet::new();
    /// set.insert("25".to_string());
    /// set.insert("40".to_string());
    /// set.insert("45".to_string());
    /// let vt = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    /// db.add("edades".to_string(), vt);
    ///
    /// assert_eq!(true, db.remove_member_from_set("edades", "40").unwrap());
    ///
    /// # let _ = std::fs::remove_file("dummy_db_rmember.csv");
    /// ```
    pub fn remove_member_from_set(&mut self, key: &str, member: &str) -> Option<bool> {
        let item = self.get_mut_live_item(key);
        match item {
            Some(item) => {
                if let ValueType::SetType(mut value) = item.get_copy_of_value() {
                    let removed = value.remove(member);
                    if removed {
                        item.set_value(ValueType::SetType(value));
                        return Some(true);
                    }
                }
            }
            None => {
                return Some(false);
            }
        }
        None
    }

    /// Actualiza el valor de tipo string almacenado en `key`.
    ///
    /// Si la clave ya contenía un valor, lo reemplaza sin importar el tipo de dato.
    ///
    /// Admite los siguientes parámetros:
    /// timeout: Actualiza el tiempo de expiración.
    /// set_if_exists: Determina si se debe actualizar la clave solo si ya existía previamente o solo si no existía previamente.
    ///
    /// Devuelve True si actualiza la clave, False si no.
    /// # Example
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    /// # use std::collections::HashSet;
    ///
    /// let mut db = Database::new("dummy_db_setstr.csv".to_string());
    /// let vt = ValueTimeItemBuilder::new(ValueType::StringType("perro".to_string())).build();
    /// db.add("mascota".to_string(), vt);
    ///
    /// assert_eq!(true, db.set_string("mascota", "gato", (&"ex".to_string(), None), Some(&"xx".to_string())));
    ///
    /// # let _ = std::fs::remove_file("dummy_db_setstr.csv");
    /// ```
    pub fn set_string(
        &mut self,
        key: &str,
        value: &str,
        timeout: (&String, Option<&String>),
        set_if_exists: Option<&String>,
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
                    let new_item = ValueTimeItemBuilder::new(value)
                        .with_key_access_time(time)
                        .build();
                    self.add(key.to_string(), new_item);
                    return true;
                }
            }
        }
        false
    }

    /// Obtiene el tiempo de expiración a partir de un offset.
    ///
    /// Admite los siguientes parámetros:
    /// * EX: Tiempo de expiración en segundos.
    /// * PX: Tiempo de expiración en milisegundos.
    /// * EXAT: Tiempo UNIX en que va a expirar la clave, en segundos.
    /// * PXAT: Tiempo UNIX en que va a expirar la clave, en milisegundos.
    ///
    /// # Example
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    ///
    /// let mut db = Database::new("dummy_db_getexpire.csv".to_string());
    ///
    /// db.get_expire_at((&"ex".to_string(), Some(&"10".to_string())));
    ///
    /// # let _ = std::fs::remove_file("dummy_db_getexpire.csv");
    /// ```
    pub fn get_expire_at(&mut self, timeout: (&String, Option<&String>)) -> u64 {
        let mut expire_at = 0;
        match timeout.0.as_str() {
            "ex" => {
                if let Some(ex) = timeout.1 {
                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("SystemTime clock failure");
                    expire_at = ex.parse::<u64>().unwrap_or(0) + now.as_secs();
                }
            }
            "px" => {
                if let Some(px) = timeout.1 {
                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("SystemTime clock failure");
                    expire_at = px.parse::<u64>().unwrap_or(0) / 1000 + now.as_millis() as u64;
                }
            }
            "exat" => {
                if let Some(exat) = timeout.1 {
                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("SystemTime clock failure");
                    expire_at = exat.parse::<u64>().unwrap_or(0) + now.as_secs();
                }
            }
            "pxat" => {
                if let Some(pxat) = timeout.1 {
                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("SystemTime clock failure");
                    expire_at = pxat.parse::<u64>().unwrap_or(0) / 1000 + now.as_secs();
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
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_pop.csv".to_string());
    /// let mut list = vec![String::from("argentina"), String::from("brasil"), String::from("chile"), String::from("uruguay")];
    /// let vt = ValueTimeItemBuilder::new(ValueType::ListType(list)).build();
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
        if let Some(item) = self.get_mut_live_item(key) {
            match item.get_copy_of_value() {
                ValueType::ListType(mut list) => {
                    let popped_elements;
                    if count < list.len() {
                        popped_elements = list.drain(..count).collect();
                    } else {
                        popped_elements = list.drain(..list.len()).collect();
                    }
                    item.set_value(ValueType::ListType(list));
                    Some(popped_elements)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// Elimina y retorna los últimos elementos de la lista almacenada en `key`.
    ///
    /// Por defecto, elimina el último elemento de la lista. Si se le pasa el parámetro opcional `count`, elimina
    /// los últimos `count` elementos.
    /// Si la clave no existe, retorna `None`. Si existe, retorna una lista con los elementos eliminados.
    /// # Examples
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_rpop.csv".to_string());
    /// let mut list = vec![String::from("argentina"), String::from("brasil"), String::from("chile"), String::from("uruguay")];
    /// let vt = ValueTimeItemBuilder::new(ValueType::ListType(list)).build();
    /// db.add("paises".to_string(), vt);
    ///
    /// let removed = db.rpop_elements_from_list("paises", 1).unwrap();
    /// assert_eq!(removed, vec![String::from("uruguay")]);
    ///
    /// let removed = db.rpop_elements_from_list("paises", 2).unwrap();
    /// assert_eq!(removed, vec![String::from("chile"), String::from("brasil")]);
    ///
    /// let _ = std::fs::remove_file("dummy_db_rpop.csv");
    /// ```
    pub fn rpop_elements_from_list(&mut self, key: &str, count: usize) -> Option<Vec<String>> {
        let mut popped_elements = Vec::new();
        if let Some(item) = self.get_mut_live_item(key) {
            if let ValueType::ListType(mut list) = item.get_copy_of_value() {
                for _ in 0..count {
                    match list.pop() {
                        Some(last_element) => {
                            popped_elements.push(last_element);
                        }
                        None => break,
                    }
                }
                item.set_value(ValueType::ListType(list));
            } else {
                return None;
            }
        } else {
            return None;
        }
        Some(popped_elements)
    }

    /// Agrega un elemento al set almacenado en `key`.
    ///
    /// Si la clave no existe, la crea.
    /// Si el valor almacenado en `key` no es de tipo Set, devuelve None.
    /// Devuelve la cantidad de elementos añadidos.
    /// # Example
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    /// # use std::collections::HashSet;
    ///
    /// let mut db = Database::new("dummy_db_sadd.csv".to_string());
    /// let mut set = HashSet::new();
    /// set.insert("25".to_string());
    /// set.insert("40".to_string());
    /// let vt = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
    /// db.add("edades".to_string(), vt);
    ///
    /// assert_eq!(2, db.add_element_to_set("edades", vec![&"45".to_string(), &"15".to_string()]).unwrap());
    ///
    /// # let _ = std::fs::remove_file("dummy_db_sadd.csv");
    /// ```
    pub fn add_element_to_set(&mut self, key: &str, values_to_add: Vec<&String>) -> Option<usize> {
        let mut added = 0;
        if let Some(item) = self.get_mut_live_item(key) {
            if let ValueType::SetType(mut old_value) = item.get_copy_of_value() {
                values_to_add.iter().for_each(|element| {
                    added += old_value.insert(element.to_string()) as usize;
                });
                item.set_value(ValueType::SetType(old_value));
                Some(added)
            } else {
                None
            }
        } else {
            let mut set = HashSet::new();
            values_to_add.iter().for_each(|element| {
                added += set.insert(element.to_string()) as usize;
            });
            let vti = ValueTimeItemBuilder::new(ValueType::SetType(set)).build();
            self.add(key.to_string(), vti);
            Some(added)
        }
    }

    /// Lee los datos del archivo de base de datos.
    ///
    /// Lee las lineas del archivo y las transforma a un KeyValueItem. Almacena estos datos en el HashMap `items`.
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    /// # use std::collections::HashSet;
    ///
    /// let mut db = Database::new("dummy_db_load.csv".to_string());
    /// db.load_items();
    /// ```
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

    /// Lee las lineas del archivo `filename`.
    ///
    /// Devuelve un iterador con las lineas leidas del archivo.
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
                let file_op = File::open(&path).expect("Could not open new database file");
                Ok(io::BufReader::new(file_op).lines())
            }
        }
    }

    /// Guarda cada item que tiene en memoria, en el formato adecuado para la serialización.
    ///
    /// El formato es: key;last_access_time;timeout;type;value
    pub fn save_items_to_file(&self) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(false)
            .create(true)
            .truncate(true)
            .open(self.dbfilename.to_string())
            .expect("Could not open database file");

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
            .unwrap_or_else(|_| println!("Could not write to database file"));
        }
    }

    /// Devuelve la cantidad de claves almacenadas en la base de datos
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    /// # use std::collections::HashSet;
    ///
    /// let mut db = Database::new("dummy_db_size.csv".to_string());
    /// db.add("mascota".to_string(),  ValueTimeItemBuilder::new(ValueType::StringType("perro".to_string())).build());
    /// db.add("animal".to_string(),  ValueTimeItemBuilder::new(ValueType::StringType("leon".to_string())).build());
    ///
    /// assert_eq!(2, db.get_size());
    ///
    /// # let _ = std::fs::remove_file("dummy_db_size.csv");
    /// ```
    pub fn get_size(&self) -> usize {
        self.items.len()
    }

    /// Elimina una clave y su valor asociado.
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    /// # use std::collections::HashSet;
    ///
    /// let mut db = Database::new("dummy_db_del.csv".to_string());
    /// db.add("mascota".to_string(),  ValueTimeItemBuilder::new(ValueType::StringType("perro".to_string())).build());
    /// db.add("animal".to_string(),  ValueTimeItemBuilder::new(ValueType::StringType("leon".to_string())).build());
    ///
    /// assert_eq!(true, db.delete_key("animal".to_string()));
    /// assert_eq!(1, db.get_size());
    ///
    /// # let _ = std::fs::remove_file("dummy_db_del.csv");
    /// ```
    pub fn delete_key(&mut self, key: String) -> bool {
        matches!(self.items.remove(&key), Some(_key))
    }

    /// Asigna un timestamp de expiración a una determinada key.
    ///
    /// Si la clave no existe, devuelve false. Si el update fue correctamente generado devuelve true.
    /// # Ejemplo
    /// ```
    /// # use proyecto_taller_1::domain::implementations::database::Database;
    /// # use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    /// # use std::collections::HashSet;
    ///
    /// let mut db = Database::new("dummy_db_expire.csv".to_string());
    /// db.add("mascota".to_string(),  ValueTimeItemBuilder::new(ValueType::StringType("perro".to_string())).build());
    ///
    /// assert_eq!(true, db.expire_key("mascota", "1623433677"));
    ///
    /// # let _ = std::fs::remove_file("dummy_db_expire.csv");
    /// ```
    pub fn expire_key(&mut self, key: &str, timeout: &str) -> bool {
        let kvi = self.get_mut_live_item(key);
        match kvi {
            Some(k) => k.set_timeout(KeyAccessTime::Volatile(u64::from_str(timeout).unwrap_or(0))),
            None => false,
        }
    }

    /// Elimina la primer `cantidad_maxima` de ocurrencias de elementos de la lista almacenada en `key`.
    ///
    /// Si la cantidad es mayor al largo de la lista, elimina elementos iguales al indicado comenzando desde el inicio de la lista.
    ///
    /// Si la cantidad es menor al largo de la lista, elimina elementos iguales al indicado comenzando desde el final de la lista.
    ///
    /// Si la cantidad es igual al largo de la lista, elimina todos los elementos iguales al indicado.
    ///
    /// Devuelve la cantidad de elementos eliminados.
    /// # Ejemplo
    /// ```
    /// use proyecto_taller_1::domain::implementations::database::Database;
    /// use proyecto_taller_1::domain::entities::key_value_item::{ValueTimeItem, ValueType, KeyAccessTime, ValueTimeItemBuilder};
    ///
    /// let mut db = Database::new("dummy_db_ldel.csv".to_string());
    /// let mut list = vec![String::from("argentina"), String::from("brasil"), String::from("argentina"), String::from("uruguay")];
    /// let vt = ValueTimeItemBuilder::new(ValueType::ListType(list)).build();
    /// db.add("paises".to_string(), vt);
    ///
    /// let removed = db.delete_elements_of_value_list("paises", "4".to_string(), "argentina".to_string());
    /// assert_eq!(removed, 2);
    ///
    /// let mut list = vec![String::from("argentina"), String::from("brasil"), String::from("argentina"), String::from("uruguay")];
    /// let vt = ValueTimeItemBuilder::new(ValueType::ListType(list)).build();
    /// db.add("paises2".to_string(), vt);
    ///
    /// let removed = db.delete_elements_of_value_list("paises2", "1".to_string(), "argentina".to_string());
    /// assert_eq!(removed, 1);
    ///
    /// let _ = std::fs::remove_file("dummy_db_ldel.csv");
    /// ```
    pub fn delete_elements_of_value_list(
        &mut self,
        key: &str,
        cantidad_maxima: String,
        element: String,
    ) -> usize {
        let mut cant_elementos_eliminados = 0;
        let cant_max = cantidad_maxima.parse::<isize>().unwrap_or(1);
        if self.key_exists(key.to_string()) {
            let old_item = self.get_mut_live_item(&key.to_string()).unwrap();
            let item_optional = old_item.get_value();
            if let ValueType::ListType(mut items) = item_optional.to_owned() {
                let len_value_list = items.len();
                match cant_max.cmp(&0) {
                    Ordering::Greater => {
                        let mut index = 0;
                        for item in items.to_owned() {
                            if cant_elementos_eliminados == cant_max {
                                break;
                            }
                            if item == element {
                                items.remove(index);
                                cant_elementos_eliminados += 1;
                            } else {
                                index += 1;
                            }
                        }
                    }
                    Ordering::Less => {
                        let mut index = len_value_list - 1;
                        for item in items.to_owned().into_iter().rev() {
                            if cant_elementos_eliminados == cant_max.abs() {
                                break;
                            } else {
                                if item == element {
                                    items.remove(index);
                                    cant_elementos_eliminados += 1;
                                };
                                if index > 0 {
                                    index -= 1;
                                } else {
                                    index = 0
                                }
                            }
                        }
                    }
                    Ordering::Equal => {
                        items.retain(|x| *x != element);
                        cant_elementos_eliminados =
                            (len_value_list as isize) - (items.len() as isize);
                    }
                }
                old_item.set_value(ValueType::ListType(items));
            }
        }
        cant_elementos_eliminados as usize
    }
}

#[test]
fn test_000_filter_keys_by_pattern() {
    let mut db = Database::new(String::from("./src/dummy_00.txt"));

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("valor_1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::StringType("valor_2".to_string())).build();
    let vt_3 = ValueTimeItemBuilder::new(ValueType::StringType("valor_3".to_string())).build();
    let vt_4 = ValueTimeItemBuilder::new(ValueType::StringType("valor_4".to_string())).build();
    db.items.insert("weight_bananas".to_string(), vt_1);
    db.items.insert("apples_weight".to_string(), vt_2);
    db.items
        .insert("deliciosos_kiwi_weight_baratos".to_string(), vt_3);
    db.items.insert("banana_weight".to_string(), vt_4);

    let vec_filtered = db.get_keys_that_match_pattern("*weight*");
    assert_eq!(vec_filtered.len(), 4);
    let _ = std::fs::remove_file("./src/dummy_00.txt");
}

#[test]
fn test_001_empty_database_returns_cero() {
    let db = Database {
        dbfilename: "file".to_string(),
        items: HashMap::new(),
    };

    assert_eq!(db.get_size(), 0);
}

#[test]
fn test_002_database_copies_value_to_new_key() {
    let mut db = Database::new(String::from("./src/dummy.txt"));
    db.add(
        "clave_1".to_string(),
        ValueTimeItemBuilder::new(ValueType::StringType("valor_1".to_string())).build(),
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
fn test_003_database_copy_replaces_key_with_new_value() {
    let mut db = Database::new(String::from("./src/dummy2.txt"));
    db.add(
        "clave_1".to_string(),
        ValueTimeItemBuilder::new(ValueType::StringType("valor_1".to_string())).build(),
    );
    db.add(
        "clave_2".to_string(),
        ValueTimeItemBuilder::new(ValueType::StringType("valor_2".to_string())).build(),
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
fn test_004_clean_items_deletes_all_items() {
    let mut db = Database::new(String::from("./src/database1.txt"));
    db.clean_items();
    assert_eq!(db.get_size(), 0);
    std::fs::remove_file("./src/database1.txt").unwrap();
}

#[test]
fn test_005_deletes_an_item_succesfully() {
    let mut db = Database::new("file2".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("valor_1".to_string()))
        .with_timeout(0)
        .build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::StringType("valor_2".to_string()))
        .with_timeout(0)
        .build();
    db.items.insert("weight_bananas".to_string(), vt_1);
    db.items.insert("apples_weight".to_string(), vt_2);

    db.delete_key("apples_weight".to_string());

    assert_eq!(db.get_size(), 1);
    std::fs::remove_file("file2").unwrap();
}

#[test]
fn test_006_persist_changes_type_of_access_time() {
    use crate::domain::entities::key_value_item::KeyAccessTime;
    let mut db = Database::new("file".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("valor_1".to_string()))
        .with_timeout(1825601548)
        .build();

    let vt_2 = ValueTimeItemBuilder::new(ValueType::StringType("valor_2".to_string()))
        .with_timeout(1825601548)
        .build();
    db.items.insert("weight_bananas".to_string(), vt_1);
    db.items.insert("apples_weight".to_string(), vt_2);

    let _res = db.persist("weight_bananas".to_string());

    let item = db.items.get(&"weight_bananas".to_string()).unwrap();
    match *item.get_timeout() {
        KeyAccessTime::Persistent => assert!(true),
        KeyAccessTime::Volatile(_tmt) => assert!(false),
    }

    std::fs::remove_file("file").unwrap();
}

#[test]
fn test_007_add_item() {
    let mut db = Database {
        dbfilename: "file".to_string(),
        items: HashMap::new(),
    };
    db.add(
        String::from("nueva_key"),
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("222"))).build(),
    );

    assert_eq!(
        db.items.get("nueva_key").unwrap().get_value().to_string(),
        String::from("222")
    );
    assert_eq!(db.items.len(), 1)
}

#[test]
fn test_008_delete_item() {
    let mut db = Database {
        dbfilename: "file".to_string(),
        items: HashMap::new(),
    };
    db.items.insert(
        String::from("nueva_key"),
        ValueTimeItemBuilder::new(ValueType::StringType(String::from("222"))).build(),
    );

    assert_eq!(db.items.len(), 1);
    db.delete_key(String::from("nueva_key"));
    assert_eq!(db.items.len(), 0);
}

#[test]
fn test_009_filename_is_correct() {
    let db = Database {
        dbfilename: "file".to_string(),
        items: HashMap::new(),
    };
    assert_eq!(db.get_filename(), &"file".to_string());
}

#[test]
fn test_010_load_items_from_file() {
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
fn test_011_create_database_file() {
    assert!(!std::path::Path::new("new_file").exists());
    let _db = Database::new("new_file".to_string());
    assert!(std::path::Path::new("new_file").exists());
    let _ = std::fs::remove_file("new_file");
}

#[test]
fn test_012_save_items_to_file() {
    use std::io::BufReader;

    let mut db = Database::new("file_save".to_string());

    let list = vec![
        "un_item_string".to_string(),
        "segundo_item_list_string".to_string(),
    ];

    db.items.insert(
        "clave_2".to_string(),
        ValueTimeItemBuilder::new(ValueType::ListType(list))
            .with_timeout(1231230)
            .build(),
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

    let _ = std::fs::remove_file("file_save");
}

#[test]
fn test_013_size_in_memory_is_correct() {
    let mut db = Database::new("file013".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("valor_1".to_string()))
        .with_timeout(0)
        .build();

    let vt_2 = ValueTimeItemBuilder::new(ValueType::StringType("valor_2".to_string()))
        .with_timeout(0)
        .build();
    db.items.insert("weight_bananas".to_string(), vt_1);
    db.items.insert("apples_weight".to_string(), vt_2);
    std::fs::remove_file("file013").unwrap();
}

#[test]
fn test_014_persist_changes_type_of_access_time() {
    use crate::domain::entities::key_value_item::KeyAccessTime;
    let mut db = Database::new(String::from("./src/dummy_persist.txt"));
    let _res = db.items.insert(
        "clave_1".to_string(),
        ValueTimeItemBuilder::new(ValueType::StringType("value".to_string())).build(),
    );

    let item = db.items.get("clave_1").unwrap();
    match *item.get_timeout() {
        KeyAccessTime::Persistent => assert!(true),
        KeyAccessTime::Volatile(_tmt) => assert!(false),
    }
    std::fs::remove_file("./src/dummy_persist.txt".to_string()).unwrap();
}

#[test]
fn test_015_append_adds_string_to_end_of_existing_value() {
    let mut db = Database::new(String::from("./src/dummy_appends_2.txt"));
    let _res = db.items.insert(
        "mykey".to_string(),
        ValueTimeItemBuilder::new(ValueType::StringType("Hello".to_string())).build(),
    );

    let len = db.append_string(&"mykey".to_string(), &" World".to_string());
    assert_eq!(len, 11);
    std::fs::remove_file("./src/dummy_appends_2.txt".to_string()).unwrap();
}

#[test]
fn test_016_append_adds_string_to_new_value() {
    let mut db = Database::new(String::from("./src/dummy_appends_1.txt"));

    let len = db.append_string(&"mykey".to_string(), &" World".to_string());
    assert_eq!(len, 6);
    std::fs::remove_file("./src/dummy_appends_1.txt".to_string()).unwrap();
}

#[test]
fn test_017_decr_key_to_existing_key() {
    let mut db = Database::new(String::from("./src/dummy_decr_1.txt"));
    let _res = db.items.insert(
        "mykey".to_string(),
        ValueTimeItemBuilder::new(ValueType::StringType("10".to_string())).build(),
    );

    let res = db.decrement_key_by(&"mykey".to_string(), 3).unwrap();
    assert_eq!(res, 7);
    std::fs::remove_file("./src/dummy_decr_1.txt".to_string()).unwrap();
}

#[test]
fn test_018_decr_by_to_new_key() {
    let mut db = Database::new(String::from("./src/dummy_decr.txt"));

    let res = db.decrement_key_by(&"mykey".to_string(), 3).unwrap();
    assert_eq!(res, -3);
    std::fs::remove_file("./src/dummy_decr.txt".to_string()).unwrap();
}

#[test]
fn test_019_decr_by_to_invalid_string_value() {
    let mut db = Database::new(String::from("./src/dummy_decr_2.txt"));
    let _res = db.items.insert(
        "mykey".to_string(),
        ValueTimeItemBuilder::new(ValueType::StringType("Hello".to_string())).build(),
    );

    let res = db.decrement_key_by(&"mykey".to_string(), 3);
    assert!(res.is_err());
    let _ = std::fs::remove_file("./src/dummy_decr_2.txt".to_string());
}

#[test]
fn test_020_se_obtienen_valores_de_claves_externas_a_partir_de_un_patron_y_una_lista_de_elementos()
{
    let mut db = Database::new("file020".to_string());

    let fruits = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "sandia".to_string(),
        "pear".to_string(),
        "apples".to_string(),
    ]))
    .build();

    db.items.insert("frutas".to_string(), fruits);

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::StringType("2".to_string())).build();
    let vt_3 = ValueTimeItemBuilder::new(ValueType::StringType("11".to_string())).build();
    let vt_4 = ValueTimeItemBuilder::new(ValueType::StringType("5".to_string())).build();

    db.items.insert("weight_bananas".to_string(), vt_1);
    db.items.insert("weight_apples".to_string(), vt_2);
    db.items.insert("weight_kiwi".to_string(), vt_3);
    db.items.insert("weight_pear".to_string(), vt_4);

    let (tuplas, _) =
        db.get_values_of_keys_matching_pattern("weight_*".to_string(), "frutas".to_string());

    assert!(tuplas.contains(&("pear".to_string(), "5".to_string())));
    assert!(tuplas.contains(&("apples".to_string(), "2".to_string())));
    let _removed = std::fs::remove_file("file020".to_string());
}

#[test]
fn test_021_se_obtienen_keys_que_contienen_patron_regex_con_signo_de_pregunta() {
    let mut db = Database::new("file021".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::StringType("2".to_string())).build();
    let vt_3 = ValueTimeItemBuilder::new(ValueType::StringType("11".to_string())).build();
    let vt_4 = ValueTimeItemBuilder::new(ValueType::StringType("5".to_string())).build();
    let vt_5 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_6 = ValueTimeItemBuilder::new(ValueType::StringType("2".to_string())).build();
    let vt_7 = ValueTimeItemBuilder::new(ValueType::StringType("11".to_string())).build();
    let vt_8 = ValueTimeItemBuilder::new(ValueType::StringType("5".to_string())).build();
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
    assert!(matching_keys.contains(&String::from("meriana")));
    assert!(matching_keys.contains(&String::from("mariana")));
    assert!(matching_keys.contains(&String::from("miriana")));
    let _removed = std::fs::remove_file("file021".to_string());
}

#[test]
fn test_022_se_obtienen_keys_que_contienen_patron_regex_solo_exp_entre_corchetes() {
    let mut db = Database::new("file022".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::StringType("2".to_string())).build();
    let vt_3 = ValueTimeItemBuilder::new(ValueType::StringType("11".to_string())).build();
    let vt_4 = ValueTimeItemBuilder::new(ValueType::StringType("5".to_string())).build();
    let vt_5 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();

    let vt_6 = ValueTimeItemBuilder::new(ValueType::StringType("2".to_string())).build();

    let vt_7 = ValueTimeItemBuilder::new(ValueType::StringType("11".to_string())).build();

    let vt_8 = ValueTimeItemBuilder::new(ValueType::StringType("5".to_string())).build();

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

    assert!(matching_keys.contains(&String::from("meriana")));
    assert!(matching_keys.contains(&String::from("mariana")));

    let _removed = std::fs::remove_file("file022".to_string());
}

#[test]
fn test_023_se_obtienen_keys_que_contienen_patron_regex_excepto_exp_entre_corchetes_tipo_1() {
    let mut db = Database::new("file023".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::StringType("2".to_string())).build();
    let vt_3 = ValueTimeItemBuilder::new(ValueType::StringType("11".to_string())).build();
    let vt_4 = ValueTimeItemBuilder::new(ValueType::StringType("5".to_string())).build();
    let vt_5 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_6 = ValueTimeItemBuilder::new(ValueType::StringType("2".to_string())).build();
    let vt_7 = ValueTimeItemBuilder::new(ValueType::StringType("11".to_string())).build();
    let vt_8 = ValueTimeItemBuilder::new(ValueType::StringType("5".to_string())).build();
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

    assert!(matching_keys.contains(&"mariana".to_string()));
    assert!(matching_keys.len() == 1);

    let _removed = std::fs::remove_file("file023".to_string());
}

#[test]
fn test_024_se_obtienen_keys_que_contienen_patron_regex_excepto_exp_entre_corchetes_tipo_2_rango() {
    let mut db = Database::new("file024".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::StringType("2".to_string())).build();

    let vt_3 = ValueTimeItemBuilder::new(ValueType::StringType("11".to_string())).build();
    let vt_4 = ValueTimeItemBuilder::new(ValueType::StringType("5".to_string())).build();
    let vt_5 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_6 = ValueTimeItemBuilder::new(ValueType::StringType("2".to_string())).build();
    let vt_7 = ValueTimeItemBuilder::new(ValueType::StringType("11".to_string())).build();
    let vt_8 = ValueTimeItemBuilder::new(ValueType::StringType("5".to_string())).build();
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

    assert!(matching_keys.contains(&String::from("meriana")));
    assert!(matching_keys.contains(&String::from("mariana")));
    assert!(matching_keys.contains(&String::from("miriana")));
    assert!(matching_keys.contains(&String::from("moriana")));
    let _ = std::fs::remove_file("file024".to_string());
}

#[test]
fn test_025_se_obtienen_keys_que_contienen_patron_regex_asterisco() {
    let mut db = Database::new("file025".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::StringType("2".to_string())).build();

    let vt_3 = ValueTimeItemBuilder::new(ValueType::StringType("11".to_string())).build();
    let vt_4 = ValueTimeItemBuilder::new(ValueType::StringType("5".to_string())).build();

    let vt_5 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_6 = ValueTimeItemBuilder::new(ValueType::StringType("2".to_string())).build();
    let vt_7 = ValueTimeItemBuilder::new(ValueType::StringType("11".to_string())).build();
    let vt_8 = ValueTimeItemBuilder::new(ValueType::StringType("5".to_string())).build();

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

    assert!(matching_keys.contains(&String::from("miriana")));
    assert!(matching_keys.contains(&String::from("meriana")));
    assert!(matching_keys.contains(&String::from("moriana")));
    assert!(matching_keys.contains(&String::from("mariana")));
    assert!(matching_keys.contains(&String::from("malala")));
    assert!(matching_keys.contains(&String::from("mia")));

    let _ = std::fs::remove_file("file025".to_string());
}

#[test]
fn test_026_expire_key() {
    let mut db = Database::new("file026".to_string());
    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string()))
        .with_timeout(1825601548)
        .build();
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
    let _ = std::fs::remove_file("file026".to_string());
}

#[test]
fn test_027_reboot_time() {
    let mut db = Database::new("file027".to_string());
    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string()))
        .with_timeout(1925583652)
        .with_last_access_time(u64::from_str("1211111").unwrap())
        .build();
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

    let _ = std::fs::remove_file("file027".to_string());
}
#[test]
fn test_028_reboot_time_expired() {
    let mut db = Database::new("file028".to_string());
    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string()))
        .with_timeout(12123120)
        .with_last_access_time(u64::from_str("1211111").unwrap())
        .build();
    db.items.insert("key123".to_string(), vt_1);
    let old_access_time = db.items.get("key123").unwrap().get_last_access_time();
    assert_eq!(old_access_time, &u64::from_str("1211111").unwrap());

    if let None = db.reboot_time("key123".to_string()) {
        assert!(true)
    } else {
        assert!(false)
    }

    let _ = std::fs::remove_file("file028".to_string());
}

#[test]
fn test_029_expired_passive_keys() {
    let mut db = Database::new("file029".to_string());
    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string()))
        .with_timeout(1625326138)
        .build();
    db.items.insert("key123".to_string(), vt_1);

    assert!(db.items.get("key123").is_some());
    let item_expired = db.get_live_item(&"key123".to_string());
    match item_expired {
        Some(_) => assert!(false),
        None => assert!(true),
    }
    assert!(db.items.get("key123").is_none());
    let _ = std::fs::remove_file("file029".to_string());
}

#[test]
fn test_030_retrieve_live_keys() {
    let mut db = Database::new("file030".to_string());
    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string()))
        .with_timeout(1665326138)
        .build();
    db.items.insert("key123".to_string(), vt_1);

    assert!(db.items.get("key123").is_some());
    let item_expired = db.get_live_item(&"key123".to_string());
    match item_expired {
        Some(_) => assert!(true),
        None => assert!(false),
    }
    assert!(db.items.get("key123").is_some());
    let _ = std::fs::remove_file("file030".to_string());
}

#[test]
fn test_031_se_obtienen_las_claves_que_contienen_solo_string_values() {
    use std::collections::HashSet;

    let mut db = Database::new("file031".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("hola".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::StringType("chau".to_string())).build();
    let vt_3 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "hola".to_string(),
        "chau".to_string(),
    ]))
    .build();
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());
    this_set.insert("value_2".to_string());
    let vt_4 = ValueTimeItemBuilder::new(ValueType::SetType(this_set))
        .with_timeout(0)
        .build();
    db.add("saludo".to_string(), vt_1);
    db.add("despido".to_string(), vt_2);
    db.add("saludo_despido".to_string(), vt_3);
    db.add("valores".to_string(), vt_4);

    let aux = db.get_string_value_by_key("saludo").unwrap();
    assert_eq!(aux, String::from("hola"));
    let aux = db.get_string_value_by_key("despido").unwrap();
    assert_eq!(aux, String::from("chau"));
    let aux = db.get_string_value_by_key("saludo_despido");
    assert!(aux.is_err());
    let aux = db.get_string_value_by_key("valores");
    assert!(aux.is_err());
    let _ = std::fs::remove_file("file031".to_string());
}

#[test]
fn test_032_scard_de_set_existente_devuelve_cantidad_de_elementos() {
    use std::collections::HashSet;
    let mut db = Database::new("file032".to_string());
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());
    this_set.insert("value_2".to_string());

    let vt = ValueTimeItemBuilder::new(ValueType::SetType(this_set)).build();

    db.items.insert("valores".to_string(), vt);
    let len = db.get_len_of_set("valores");
    assert_eq!(len, 2);
    let _ = std::fs::remove_file("file032".to_string());
}

#[test]
fn test_033_scard_de_set_devuelve_cero_si_no_existe() {
    let db = Database::new("file033".to_string());

    let len = db.get_len_of_set("valores");
    assert_eq!(len, 0);
    let _ = std::fs::remove_file("file033".to_string());
}

#[test]
fn test_034_scard_de_set_devuelve_cero_si_no_es_tipo_set() {
    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("hola".to_string()))
        .with_timeout(0)
        .build();
    let mut db = Database::new("file034".to_string());
    db.items.insert("saludo".to_string(), vt_1);

    let len = db.get_len_of_set("saludo");
    assert_eq!(len, 0);
    let _ = std::fs::remove_file("file034".to_string());
}

#[test]
fn test_035_ismember_de_set_devuelve_cero_si_no_es_tipo_set() {
    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("hola".to_string()))
        .with_timeout(0)
        .build();

    let mut db = Database::new("file035".to_string());
    db.items.insert("saludo".to_string(), vt_1);

    let len = db.is_member_of_set("saludo", "hola");
    assert_eq!(len, 0);
    let _ = std::fs::remove_file("file035".to_string());
}

#[test]
fn test_036_ismember_de_set_devuelve_cero_si_no_existe_clave() {
    let db = Database::new("file036".to_string());

    let len = db.is_member_of_set("valores", "hola");
    assert_eq!(len, 0);
    let _ = std::fs::remove_file("file036".to_string());
}

#[test]
fn test_037_ismember_de_set_existente_devuelve_uno() {
    use std::collections::HashSet;
    let mut db = Database::new("file037".to_string());
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());

    let vt = ValueTimeItemBuilder::new(ValueType::SetType(this_set)).build();

    db.items.insert("valores".to_string(), vt);
    let is_member = db.is_member_of_set("valores", "value_1");
    assert_eq!(is_member, 1);
    let _ = std::fs::remove_file("file037".to_string());
}

#[test]
fn test_038_ismember_de_set_existente_devuelve_cero_si_no_pertenece_al_set() {
    use std::collections::HashSet;
    let mut db = Database::new("file038".to_string());
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());

    let vt = ValueTimeItemBuilder::new(ValueType::SetType(this_set)).build();

    db.items.insert("valores".to_string(), vt);
    let is_member = db.is_member_of_set("valores", "value_2");
    assert_eq!(is_member, 0);
    let _ = std::fs::remove_file("file038".to_string());
}

#[test]
fn test_039_get_members_of_set_existente_devuelve_elementos_del_set() {
    use std::collections::HashSet;
    let mut db = Database::new("file039".to_string());
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());
    this_set.insert("value_2".to_string());
    this_set.insert("value_3".to_string());

    let vt = ValueTimeItemBuilder::new(ValueType::SetType(this_set)).build();

    db.items.insert("valores".to_string(), vt);
    let members = db.get_members_of_set("valores");
    assert!(members.contains(&&String::from("value_1")));
    assert!(members.contains(&&String::from("value_2")));
    assert!(members.contains(&&String::from("value_3")));
    assert_eq!(members.len(), 3);
    let _ = std::fs::remove_file("file039".to_string());
}

#[test]
fn test_040_remove_member_from_existing_set_returns_true() {
    use std::collections::HashSet;
    let mut db = Database::new("file040".to_string());
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());
    this_set.insert("value_2".to_string());

    let vt = ValueTimeItemBuilder::new(ValueType::SetType(this_set)).build();

    db.items.insert("valores".to_string(), vt);
    let removed = db.remove_member_from_set("valores", "value_1").unwrap();
    assert_eq!(removed, true);

    let _ = std::fs::remove_file("file040".to_string());
}

#[test]
fn test_041_remove_member_from_non_existing_set_returns_false() {
    use std::collections::HashSet;
    let mut db = Database::new("file041".to_string());
    let mut this_set = HashSet::new();
    this_set.insert("value_1".to_string());
    this_set.insert("value_2".to_string());

    let vt = ValueTimeItemBuilder::new(ValueType::SetType(this_set)).build();

    db.items.insert("values".to_string(), vt);
    let removed = db.remove_member_from_set("valores", "value_1").unwrap();
    assert_eq!(removed, false);

    let _ = std::fs::remove_file("file041".to_string());
}

#[test]
fn test_042_remove_member_from_list_type_returns_none() {
    let mut db = Database::new("file042".to_string());
    let vt = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "hola".to_string(),
        "chau".to_string(),
    ]))
    .build();
    db.items.insert("saludo".to_string(), vt);
    let removed = db.remove_member_from_set("saludo", "value_1");
    assert!(removed.is_none());

    let _ = std::fs::remove_file("file042".to_string());
}

#[test]
fn test_043_pop_one_element_from_list_returns_popped_element() {
    let mut db = Database::new("file043".to_string());
    let vt = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "hola".to_string(),
        "chau".to_string(),
    ]))
    .build();
    db.items.insert("saludo".to_string(), vt);
    let removed = db.pop_elements_from_list("saludo", 1).unwrap();
    assert_eq!(removed, vec![String::from("hola")]);
    let item = db.get_live_item("saludo").unwrap();
    if let ValueType::ListType(item) = item.get_value() {
        assert_eq!(item, &vec![String::from("chau")]);
    } else {
        assert!(false);
    }

    let _ = std::fs::remove_file("file043".to_string());
}

#[test]
fn test_044_pop_multiple_elements_from_list_returns_popped_elements() {
    let mut db = Database::new("file044".to_string());
    let vt = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "hola".to_string(),
        "chau".to_string(),
        "hello".to_string(),
        "bye".to_string(),
    ]))
    .build();
    db.items.insert("saludo".to_string(), vt);
    let removed = db.pop_elements_from_list("saludo", 2).unwrap();
    assert_eq!(removed, vec![String::from("hola"), String::from("chau")]);
    let item = db.get_live_item("saludo").unwrap();
    if let ValueType::ListType(item) = item.get_value() {
        assert!(item.contains(&String::from("hello")) && item.contains(&String::from("bye")));
    } else {
        assert!(false);
    }

    let _ = std::fs::remove_file("file044".to_string());
}

#[test]
fn test_045_rpop_one_element_from_list_returns_popped_element() {
    let mut db = Database::new("file045".to_string());
    let vt = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "hola".to_string(),
        "chau".to_string(),
    ]))
    .build();
    db.items.insert("saludo".to_string(), vt);
    let removed = db.rpop_elements_from_list("saludo", 1).unwrap();
    assert_eq!(removed, vec![String::from("chau")]);
    let item = db.get_live_item("saludo").unwrap();
    if let ValueType::ListType(item) = item.get_value() {
        assert_eq!(item, &vec![String::from("hola")]);
    } else {
        assert!(false);
    }

    let _ = std::fs::remove_file("file045".to_string());
}

#[test]
fn test_046_rpop_multiple_elements_from_list_returns_popped_elements() {
    let mut db = Database::new("file046".to_string());
    let vt = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "hola".to_string(),
        "chau".to_string(),
        "hello".to_string(),
        "bye".to_string(),
    ]))
    .build();
    db.items.insert("saludo".to_string(), vt);
    let removed = db.rpop_elements_from_list("saludo", 2).unwrap();
    assert_eq!(removed, vec![String::from("bye"), String::from("hello")]);
    let item = db.get_live_item("saludo").unwrap();
    if let ValueType::ListType(item) = item.get_value() {
        assert!(item.contains(&String::from("hola")) && item.contains(&String::from("chau")));
    } else {
        assert!(false);
    }

    let _ = std::fs::remove_file("file046".to_string());
}

#[test]
fn test_047_rpush_multiple_elements_to_list_returns_length() {
    let mut db = Database::new("file047".to_string());
    let vt = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "hola".to_string(),
        "chau".to_string(),
        "hello".to_string(),
        "bye".to_string(),
    ]))
    .build();
    db.items.insert("saludo".to_string(), vt);
    let len = db.push_vec_to_list(
        vec![String::from("salut"), String::from("au revoir")],
        "saludo",
    );
    assert_eq!(len, 6);

    let _ = std::fs::remove_file("file047".to_string());
}

#[test]
fn test_048_rpush_to_nonexisting_key_returns_zero() {
    let mut db = Database::new("file048".to_string());
    let vt = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "hola".to_string(),
        "chau".to_string(),
        "hello".to_string(),
        "bye".to_string(),
    ]))
    .build();
    db.items.insert("despido".to_string(), vt);
    let len = db.push_vec_to_list(
        vec![String::from("salut"), String::from("au revoir")],
        "saludo",
    );
    assert_eq!(len, 0);

    let _ = std::fs::remove_file("file048".to_string());
}

#[test]
fn test_049_rpush_to_string_returns_zero() {
    let mut db = Database::new("file049".to_string());
    let vt = ValueTimeItemBuilder::new(ValueType::StringType("hola".to_string())).build();
    db.items.insert("saludo".to_string(), vt);
    let len = db.push_vec_to_list(
        vec![String::from("salut"), String::from("au revoir")],
        "saludo",
    );
    assert_eq!(len, 0);

    let _ = std::fs::remove_file("file049".to_string());
}

#[test]
fn test_050_se_eliminan_3_elementos_de_value_list_type() {
    let mut db = Database::new("file050".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "my".to_string(),
        "dog".to_string(),
        "my".to_string(),
        "friend".to_string(),
        "my".to_string(),
        "family".to_string(),
        "my".to_string(),
        "friend".to_string(),
    ]))
    .build();

    db.items.insert("mia".to_string(), vt_1);
    db.items.insert("phrase".to_string(), vt_2);

    let values_deleted =
        db.delete_elements_of_value_list("phrase", "3".to_string(), "my".to_string());

    assert_eq!(3, values_deleted);

    std::fs::remove_file("file050".to_string()).unwrap();
}

#[test]
fn test_051_se_eliminan_todos_los_elementos_de_value_list_type() {
    let mut db = Database::new("file051".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "my".to_string(),
        "dog".to_string(),
        "my".to_string(),
        "friend".to_string(),
        "my".to_string(),
        "family".to_string(),
        "my".to_string(),
        "friend".to_string(),
    ]))
    .build();

    db.items.insert("mia".to_string(), vt_1);
    db.items.insert("phrase".to_string(), vt_2);

    let values_deleted =
        db.delete_elements_of_value_list("phrase", "0".to_string(), "my".to_string());

    assert_eq!(4, values_deleted);
    std::fs::remove_file("file051".to_string()).unwrap();
}

#[test]
fn test_052_se_eliminan_3_elementos_de_value_list_type_en_reversa() {
    let mut db = Database::new("file052".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "my".to_string(),
        "dog".to_string(),
        "my".to_string(),
        "friend".to_string(),
        "my".to_string(),
        "family".to_string(),
        "my".to_string(),
        "dear".to_string(),
    ]))
    .build();

    db.items.insert("mia".to_string(), vt_1);
    db.items.insert("phrase".to_string(), vt_2);
    let values_deleted =
        db.delete_elements_of_value_list("phrase", "-3".to_string(), "my".to_string());
    assert_eq!(3, values_deleted);
    std::fs::remove_file("file052".to_string()).unwrap();
}

#[test]
fn test_053_se_obtienen_3_elementos_de_un_value_de_tipo_list_clave_existe() {
    let mut db = Database::new("file053".to_string());
    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "my".to_string(),
        "dog".to_string(),
        "my".to_string(),
        "friend".to_string(),
        "my".to_string(),
        "family".to_string(),
        "my".to_string(),
        "dear".to_string(),
    ]))
    .build();

    db.items.insert("mia".to_string(), vt_1);
    db.items.insert("phrase".to_string(), vt_2);
    let elements_got = db.get_values_in_range("phrase", "2", "4").unwrap();
    assert_eq!(3, elements_got.len());
    std::fs::remove_file("file053".to_string()).unwrap();
}

#[test]
fn test_054_se_obtienen_3_elementos_de_un_value_de_tipo_list_clave_existe_con_lb_y_ub_negativos() {
    let mut db = Database::new("file054".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "my".to_string(),
        "dog".to_string(),
        "my".to_string(),
        "friend".to_string(),
        "my".to_string(),
        "family".to_string(),
        "my".to_string(),
        "dear".to_string(),
    ]))
    .build();

    db.items.insert("mia".to_string(), vt_1);
    db.items.insert("phrase".to_string(), vt_2);
    let elements_got = db.get_values_in_range("phrase", "-4", "-2").unwrap();
    assert_eq!(3, elements_got.len());
    std::fs::remove_file("file054".to_string()).unwrap();
}

#[test]
fn test_055_se_obtiene_un_vector_vacio_de_1_elemento_cuando_lb_es_mayor_que_ub() {
    let mut db = Database::new("file055".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "my".to_string(),
        "dog".to_string(),
        "my".to_string(),
        "friend".to_string(),
        "my".to_string(),
        "family".to_string(),
        "my".to_string(),
        "dear".to_string(),
    ]))
    .build();

    db.items.insert("mia".to_string(), vt_1);
    db.items.insert("phrase".to_string(), vt_2);
    let elements_got = db.get_values_in_range("phrase", "5", "2").unwrap();
    assert_eq!(1, elements_got.len());
    std::fs::remove_file("file055".to_string()).unwrap();
}

#[test]
fn test_056_se_obtiene_un_vector_de_longitud_maxima_lenght_cuando_ub_es_mayor_que_lenght() {
    let mut db = Database::new("file056".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "my".to_string(),
        "dog".to_string(),
        "my".to_string(),
        "friend".to_string(),
        "my".to_string(),
        "family".to_string(),
        "my".to_string(),
        "dear".to_string(),
    ]))
    .build();

    db.items.insert("mia".to_string(), vt_1);
    db.items.insert("phrase".to_string(), vt_2);
    let elements_got = db.get_values_in_range("phrase", "0", "200").unwrap();
    assert_eq!(8, elements_got.len());
    std::fs::remove_file("file056".to_string()).unwrap();
}

#[test]
fn test_057_se_obtiene_trozo_de_lista_de_value_de_tipo_list() {
    let mut db = Database::new("file057".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "my".to_string(),
        "dog".to_string(),
        "my".to_string(),
        "friend".to_string(),
        "my".to_string(),
        "family".to_string(),
        "my".to_string(),
        "dear".to_string(),
    ]))
    .build();

    db.items.insert("mia".to_string(), vt_1);
    db.items.insert("phrase".to_string(), vt_2);
    let trozo_value_list_type = db.get_values_in_range("phrase", "0", "2"); //("phrase", "-3".to_string(), "my".to_string());
    assert_eq!(3, trozo_value_list_type.unwrap().len());
    std::fs::remove_file("file057".to_string()).unwrap();
}

#[test]
fn test_058_se_obtiene_trozo_de_lista_de_value_de_tipo_list_lower_bound_negativo() {
    let mut db = Database::new("file058".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "my".to_string(),
        "dog".to_string(),
        "my".to_string(),
        "friend".to_string(),
        "my".to_string(),
        "family".to_string(),
        "my".to_string(),
        "dear".to_string(),
    ]))
    .build();

    db.items.insert("mia".to_string(), vt_1);
    db.items.insert("phrase".to_string(), vt_2);
    let trozo_value_list_type = db.get_values_in_range("phrase", "0", "-5"); //("phrase", "-3".to_string(), "my".to_string());
    assert_eq!(4, trozo_value_list_type.unwrap().len());
    std::fs::remove_file("file058".to_string()).unwrap();
}

#[test]
fn test_059_se_obtiene_trozo_de_lista_de_value_de_tipo_list_lower_y_upper_bound_negativos() {
    let mut db = Database::new("file059".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "my".to_string(),
        "dog".to_string(),
        "my".to_string(),
        "friend".to_string(),
        "my".to_string(),
        "family".to_string(),
        "my".to_string(),
        "dear".to_string(),
    ]))
    .build();

    db.items.insert("mia".to_string(), vt_1);
    db.items.insert("phrase".to_string(), vt_2);
    let trozo_value_list_type = db.get_values_in_range("phrase", "-7", "-5"); //("phrase", "-3".to_string(), "my".to_string());
    assert_eq!(3, trozo_value_list_type.unwrap().len());
    std::fs::remove_file("file059".to_string()).unwrap();
}

#[test]
fn test_060_se_pisan_valores_en_value_de_tipo_list_type() {
    let mut db = Database::new("file060".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "juan".to_string(),
        "pedro".to_string(),
        "santiago".to_string(),
        "mariano".to_string(),
        "francisco".to_string(),
        "domingo".to_string(),
        "rolando".to_string(),
        "fernando".to_string(),
    ]))
    .build();

    db.items.insert("mia".to_string(), vt_1);
    db.items.insert("nombres_masculinos".to_string(), vt_2);
    let vec_actualizado =
        db.replace_element_in_list_type_value("nombres_masculinos", "sergio", "0");
    let current_value = db.get_mut_live_item("nombres_masculinos").unwrap();
    let item_optional = current_value.get_value();
    if let ValueType::ListType(items) = item_optional.to_owned() {
        assert_eq!("sergio".to_string(), items[0]);
    }
    assert_eq!(true, vec_actualizado);
    std::fs::remove_file("file060".to_string()).unwrap();
}

#[test]
fn test_061_no_se_reemplaza_valor_en_value_de_tipo_list_type_porque_fuera_de_rango() {
    let mut db = Database::new("file061".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "juan".to_string(),
        "pedro".to_string(),
        "santiago".to_string(),
        "mariano".to_string(),
        "francisco".to_string(),
        "domingo".to_string(),
        "rolando".to_string(),
        "fernando".to_string(),
    ]))
    .build();

    db.items.insert("mia".to_string(), vt_1);
    db.items.insert("nombres_masculinos".to_string(), vt_2);
    let vec_actualizado =
        db.replace_element_in_list_type_value("nombres_masculinos", "sergio", "10");
    assert_eq!(false, vec_actualizado);
    std::fs::remove_file("file061".to_string()).unwrap();
}

#[test]
fn test_062_se_pisan_valores_en_value_de_tipo_list_type_con_indice_negativo_inbound() {
    let mut db = Database::new("file062".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    let vt_2 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "juan".to_string(),
        "pedro".to_string(),
        "santiago".to_string(),
        "mariano".to_string(),
        "francisco".to_string(),
        "domingo".to_string(),
        "rolando".to_string(),
        "fernando".to_string(),
    ]))
    .build();

    db.items.insert("mia".to_string(), vt_1);
    db.items.insert("nombres_masculinos".to_string(), vt_2);
    let vec_actualizado =
        db.replace_element_in_list_type_value("nombres_masculinos", "sergio", "-1");
    let current_value = db.get_mut_live_item("nombres_masculinos").unwrap();
    let item_optional = current_value.get_value();
    if let ValueType::ListType(items) = item_optional.to_owned() {
        assert_eq!("sergio".to_string(), items[7]);
    }
    assert_eq!(true, vec_actualizado);
    std::fs::remove_file("file062".to_string()).unwrap();
}

#[test]
fn test_063_set_string_con_expire_returns_true() {
    let mut db = Database::new("file063".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    db.items.insert("mia".to_string(), vt_1);

    assert!(db.set_string(
        "mia",
        "2",
        (&"ex".to_string(), Some(&"10".to_string())),
        None
    ));

    std::fs::remove_file("file063".to_string()).unwrap();
}

#[test]
fn test_064_set_string_con_expire_and_set_only_returns_true() {
    let mut db = Database::new("file064".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    db.items.insert("mia".to_string(), vt_1);

    assert!(db.set_string(
        "mia",
        "2",
        (&"ex".to_string(), Some(&"10".to_string())),
        Some(&"xx".to_string())
    ));

    std::fs::remove_file("file064".to_string()).unwrap();
}

#[test]
fn test_065_set_string_con_expire_and_set_only_returns_false() {
    let mut db = Database::new("file065".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::StringType("1".to_string())).build();
    db.items.insert("mia".to_string(), vt_1);

    assert!(!db.set_string(
        "another",
        "2",
        (&"ex".to_string(), Some(&"10".to_string())),
        Some(&"xx".to_string())
    ));

    std::fs::remove_file("file065".to_string()).unwrap();
}

#[test]
fn test_066_add_to_list_adds_elements_from_beginning() {
    let mut db = Database::new("file066".to_string());

    let vt_1 = ValueTimeItemBuilder::new(ValueType::ListType(vec![
        "1".to_string(),
        "2".to_string(),
        "3".to_string(),
    ]))
    .build();
    db.items.insert("numbers".to_string(), vt_1);

    assert_eq!(
        db.add_to_list_type(
            vec!["primero".to_string(), "segundo".to_string()],
            "numbers",
            false
        )
        .unwrap(),
        5
    );
    let item = db.get_live_item("numbers").unwrap().get_value();
    if let ValueType::ListType(list) = item {
        assert_eq!(
            list,
            &vec![
                "primero".to_string(),
                "segundo".to_string(),
                "1".to_string(),
                "2".to_string(),
                "3".to_string()
            ]
        )
    }
    std::fs::remove_file("file066".to_string()).unwrap();
}
