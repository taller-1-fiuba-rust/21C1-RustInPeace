#[derive(Debug, PartialEq)]
pub enum RespType {
    RSimpleString(String),
    RError(String),
    RInteger(usize),
    RBulkString(String),
    RArray(Vec<RespType>),
    RNullBulkString(),
    RNullArray(),
}
