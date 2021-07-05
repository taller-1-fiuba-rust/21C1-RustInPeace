use crate::services::utils::resp_type::RespType;

#[derive(Debug)]
pub struct OperationRegister {
    operations: Vec<Vec<String>>,
    max_operations: usize,
}

impl OperationRegister {
    pub fn new(max_operations: usize) -> Self {
        let max_operations = max_operations;
        let operations = Vec::with_capacity(max_operations);
        OperationRegister {
            operations,
            max_operations,
        }
    }

    /// Guarda una operacion en el registro
    pub fn store_operation(&mut self, operation: RespType) {
        if let RespType::RArray(command_vector) = operation {
            let mut vec_aux = Vec::<String>::new();
            for element in command_vector {
                if let RespType::RBulkString(string) = element {
                    if self.operations.len() >= self.max_operations {
                        self.operations.swap_remove(0);
                    }
                    vec_aux.push(string.to_string())
                }
            }
            self.operations.push(vec_aux)
        }
    }

    /// Devuelve la lista de operariones registradas
    pub fn get_operations(&self) -> &Vec<Vec<String>> {
        &self.operations
    }
}

#[test]
fn test_01_se_guardan_vectores_de_tipo_resptype_en_field_operations() {
    let elemento_1a = RespType::RBulkString("set_a".to_string());
    let elemento_2a = RespType::RBulkString("key_a".to_string());
    let elemento_3a = RespType::RBulkString("value_a".to_string());
    let vector_aux_a = vec![elemento_1a, elemento_2a, elemento_3a];
    let vec_resp_type_a = RespType::RArray(vector_aux_a);

    let elemento_1b = RespType::RBulkString("set_b".to_string());
    let elemento_2b = RespType::RBulkString("key_b".to_string());
    let elemento_3b = RespType::RBulkString("value_b".to_string());
    let vector_aux_b = vec![elemento_1b, elemento_2b, elemento_3b];
    let vec_resp_type_b = RespType::RArray(vector_aux_b);

    let mut register = OperationRegister::new(2);
    register.store_operation(vec_resp_type_a);
    register.store_operation(vec_resp_type_b);
    let vector_of_operations = register.get_operations();

    assert_eq!(&vec![vec![String::from("set_a"), String::from("key_a"), String::from("value_a")], vec![String::from("set_b"), String::from("key_b"), String::from("value_b")]], vector_of_operations);
}

#[test]
fn test_02_se_elimina_el_primer_elemento_y_se_guarda_el_nuevo_cuando_esta_lleno() {
    let elemento_1a = RespType::RBulkString("set_a".to_string());
    let elemento_2a = RespType::RBulkString("key_a".to_string());
    let elemento_3a = RespType::RBulkString("value_a".to_string());
    let vector_aux_a = vec![elemento_1a, elemento_2a, elemento_3a];
    let vec_resp_type_a = RespType::RArray(vector_aux_a);

    let elemento_1b = RespType::RBulkString("set_b".to_string());
    let elemento_2b = RespType::RBulkString("key_b".to_string());
    let elemento_3b = RespType::RBulkString("value_b".to_string());
    let vector_aux_b = vec![elemento_1b, elemento_2b, elemento_3b];
    let vec_resp_type_b = RespType::RArray(vector_aux_b);

    let elemento_1c = RespType::RBulkString("set_c".to_string());
    let elemento_2c = RespType::RBulkString("key_c".to_string());
    let elemento_3c = RespType::RBulkString("value_c".to_string());
    let vector_aux_c = vec![elemento_1c, elemento_2c, elemento_3c];
    let vec_resp_type_c = RespType::RArray(vector_aux_c);

    let mut register = OperationRegister::new(2);
    register.store_operation(vec_resp_type_a);
    register.store_operation(vec_resp_type_b);
    register.store_operation(vec_resp_type_c);
    let vector_of_operations = register.get_operations();

    assert_eq!(&vec![vec![String::from("set_b"), String::from("key_b"), String::from("value_b")], vec![String::from("set_c"), String::from("key_c"), String::from("value_c")]], vector_of_operations);
}
