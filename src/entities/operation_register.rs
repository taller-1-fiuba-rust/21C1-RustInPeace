use crate::entities::resp_types::RespType;

#[derive(Debug)]
pub struct OperationRegister {
    operations: Vec<Vec<String>>,
    max_operations: usize,
}

impl OperationRegister {
    pub fn new() -> Self {
        let max_operations = 2;
        let operations = Vec::with_capacity(max_operations);
        OperationRegister {
            operations,
            max_operations,
        }
    }

    pub fn store_operation(&mut self, operation: RespType) {
        if let RespType::RArray(command_vector) = operation {
            let mut vec_aux = Vec::<String>::new();
            for element in command_vector {
                if let RespType::RBulkString(string) = element {
                    if self.operations.len() >= self.max_operations {
                        self.operations.swap_remove(0);
                    }
                    vec_aux.push(string)
                }
            }
            self.operations.push(vec_aux)
        }
    }

    pub fn monitor(self) -> Vec<Vec<String>> {
        self.operations
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

    let mut register = OperationRegister::new();
    register.store_operation(vec_resp_type_a);
    register.store_operation(vec_resp_type_b);
    let vector_of_operations = register.monitor();
    for elemento in vector_of_operations {
        println!("{:?}", elemento)
    }
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

    let mut register = OperationRegister::new();
    register.store_operation(vec_resp_type_a);
    register.store_operation(vec_resp_type_b);
    register.store_operation(vec_resp_type_c);
    let vector_of_operations = register.monitor();
    for elemento in vector_of_operations {
        println!("{:?}", elemento)
    }
}
