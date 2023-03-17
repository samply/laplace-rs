use thiserror::Error;

#[derive(Error, Debug)]
pub enum LaplaceError {
    #[error("Unable to create Laplace distribution")]
    DistributionCreationError(String),
    #[error("Deserialization error")]
    DeserializationError(String),
    #[error("Serialization error")]
    SerializationError(String),
}