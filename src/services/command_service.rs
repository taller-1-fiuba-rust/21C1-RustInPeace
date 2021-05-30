use crate::entities::operation_register::OperationRegister;
use crate::entities::resp_types::RespType;

pub fn register_operation(register: &mut OperationRegister, operation: RespType) {
    register.store_operation(operation)
}
