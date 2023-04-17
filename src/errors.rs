use thiserror::Error;

#[derive(Error, Debug)]
pub enum LaplaceError {
    #[error("Unable to create Laplace distribution")]
    DistributionCreationError(String),
    #[error("Rounding step error")]
    RoundingStepError(String),
}
