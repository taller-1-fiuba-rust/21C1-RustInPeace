//! Hace un mapeo entre los tipos de datos del protocolo Redis y los tipos de datos Rust.

use std::fmt::{Display, Error, Formatter};

/// Los clientes Redis se comunican con el servidor Redis a través del protocolo RESP.
/// Los tipos de datos que soporta son:
/// * Simple string
/// * Bulk string
/// * Error
/// * Array
/// * Integer
///
/// Además, admite valores nulos como una variación de bulk strings y arrays (Null bulk string y null array, respectivamente).
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

/// Implementa el Trait `Display`.
///
/// En el caso de `RArray`, muestra los elementos del vector separados por un espacio en blanco.
/// Si es un elemento nulo, muestra `(nil)`.
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
                        concatenated_elements.push(' ');
                    }
                }
                write!(f, "{}", concatenated_elements)
            }
            RespType::RNullBulkString() => write!(f, "(nil)"),
            RespType::RNullArray() => write!(f, "(nil)"),
        }
    }
}
