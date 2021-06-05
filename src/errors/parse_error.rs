#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidProtocol(String),
    InvalidSize(String),
    IntParseError(String),
    UnexpectedError(String),
    InvalidRequest(String),
}
