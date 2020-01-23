use std::io::{Error};

/// Map to serializeable representation
pub trait IntoSerializable {
    type Output;
    fn into_serializable(&self) -> Self::Output;
}

/// Map from deserialized representation
pub trait FromDeserialized: Sized {
    type Input;
    fn from_deserialized(input: Self::Input) -> Result<Self, Error>;
}