//! Errores relacionados al parseo de solicitudes.

/// Se establecen los siguientes tipos de error:
/// * InvalidProtocol
/// * InvalidSize
/// * IntParseError
/// * UnexpectedError
/// * InvalidRequest
#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidProtocol(String),
    InvalidSize(String),
    IntParseError(String),
    UnexpectedError(String),
    InvalidRequest(String),
}
