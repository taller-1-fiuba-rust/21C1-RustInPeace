use std::fmt;

#[derive(Debug, PartialEq)]
pub enum DatabaseError {
    InvalidValueType(String),
    MissingKey(),
    InvalidParameter(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error encountered while operating with database")
    }
}
