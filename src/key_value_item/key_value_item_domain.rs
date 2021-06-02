pub enum ValueType {
    List,
    Set,
    String,
}

#[derive(Debug)]
pub struct KeyValueItem {
    key: String,
    value: String,            // Extender a los tipos permitidos.
    last_access_time: u64
}

impl KeyValueItem {
    pub fn new(key: String, value: String) -> KeyValueItem {
        KeyValueItem {
            key: key.to_string(), // Es probable que esta key pueda autogenerarse
            value,
            last_access_time: 1622657604,
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn larger_can_hold_smaller() {
        unimplemented!()

    }
}