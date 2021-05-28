use crate::entities::resp_types::RESPTypes;
use crate::entities::error::ParseError;

pub trait DataType {
    fn deserialize(self) -> String;
}