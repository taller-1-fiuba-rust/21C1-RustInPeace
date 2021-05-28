#[derive(Debug, PartialEq)]
pub enum RESPTypes {
    RSimpleString(String),
    RError(String),
    RInteger(usize),
    RBulkString(String),
    RArray(Vec<RESPTypes>)
}