use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidProtocol(String),
    InvalidSize(String),
    IntParseError(String),
    UnexpectedError(String),
    InvalidRequest(String),
}

impl Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::InvalidProtocol(_) => "RESP Protocol error",
            ParseError::IntParseError(_) => "Error occured while parsing int",
            ParseError::InvalidSize(_) => "Size mismatch",
            ParseError::UnexpectedError(_) => "Unexpected error while parsing",
            ParseError::InvalidRequest(_) => "Invalid request error",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error occurred while parsing")
    }
}
