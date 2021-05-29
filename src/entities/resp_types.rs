#[derive(Debug, PartialEq)]
pub enum RespTypes {
    RSimpleString(String),
    RError(String),
    RInteger(usize),
    RBulkString(String),
    RArray(Vec<RespTypes>),
    RNullBulkString(),
    RNullArray(),
}
