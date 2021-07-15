use std::{fmt::{Display, Formatter, Error}};

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

impl Display for RespType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            RespType::RSimpleString(str) => write!(f, "{}", str),
            RespType::RError(str) => write!(f, "{}", str),
            RespType::RInteger(int) => write!(f, "{}", int),
            RespType::RSignedNumber(int) => write!(f, "{}", int),
            RespType::RBulkString(str) => write!(f, "{}", str),
            RespType::RArray(array) => {
                let mut concatenated_elements = String::new();
                for e in array {
                    if let RespType::RBulkString(str) = e {
                        concatenated_elements.push_str(str);
                        concatenated_elements.push_str(" ");
                    }
                }
                write!(f, "{}", concatenated_elements)
            },
            RespType::RNullBulkString() => write!(f, "(nil)"),
            RespType::RNullArray() => write!(f, "(nil)"),
        }
    }
}
