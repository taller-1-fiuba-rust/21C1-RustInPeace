#[derive(Debug, PartialEq, Clone)]
pub enum RespType {
    RSimpleString(String),
    RError(String),
    RInteger(usize),
    RSignedNumber(isize),
    RBulkString(String),
    RArray(Vec<RespType>),
    RNullBulkString(),
    RNullArray(),
}
