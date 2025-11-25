use statrs::distribution::LaplaceError as StatsError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LaplaceError {
    #[error("Unable to create Laplace distribution: {0}")]
    DistributionCreationError(StatsError),
    #[error("Invalid clamping domain. Must be None or non-zero positive number")]
    InvalidClamping,
    #[error("Rounding step zero not allowed")]
    InvalidArgRoundingStepZero,
    #[error("Rounding step error: {0}")]
    RoundingStepError(String),
}
