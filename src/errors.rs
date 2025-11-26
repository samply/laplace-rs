use statrs::distribution::LaplaceError as StatsError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LaplaceError {
    #[error("Unable to create Laplace distribution: {0}")]
    DistributionCreationError(StatsError),
    #[error("Invalid domain limit. Must be None or a positive non-zero number")]
    InvalidDomain,
    #[error("Rounding step zero not allowed")]
    InvalidArgRoundingStepZero,
    #[error("Rounding step error: {0}")]
    RoundingStepError(String),
}
