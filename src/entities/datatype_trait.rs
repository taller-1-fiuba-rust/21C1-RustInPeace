pub trait DataType {
    fn deserialize(self) -> String;
}