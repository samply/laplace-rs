use statrs::StatsError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LaplaceError {
    #[error("Unable to create Laplace distribution: {0}")]
    DistributionCreationError(StatsError),
    #[error("Rounding step zero not allowed")]
    InvalidArgRoundingStepZero,
    #[error("Rounding step error: {0}")]
    RoundingStepError(String),
}
