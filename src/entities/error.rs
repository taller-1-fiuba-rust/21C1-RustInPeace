//#[derive(Debug)]
//pub enum ResponseError {
//    GenericError(String)
//}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidProtocol(String),
    InvalidSize(String),
    IntParseError(String),
    UnexpectedError(String),
}
