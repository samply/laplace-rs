pub mod errors;

use anyhow::Result;
use rand::distributions::Distribution;
use statrs::distribution::Laplace;
use std::collections::HashMap;

use crate::errors::LaplaceError;

// obfuscation cache
type Sensitivity = usize;
type Count = u64;
pub type Bin = usize;
pub struct ObfCache {
    pub cache: HashMap<(Sensitivity, Count, Bin), u64>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ObfuscateBelow10Mode {
    Zero,
    Ten,
    Obfuscate,
}

/// Obfuscates the given value using a random sampled value from a Laplace distribution with
/// given delta and epsilon parameters, and bin to which the value belongs. The
/// obfuscate_zero flag indicates whether only positive values should be obfuscated or all
/// values, including zero. The rounding_step determines the granularity of the rounding. If
/// obf_cache_option is not None, the function checks the cache for a pre-computed value before
/// obfuscating. If no cached value is found, it obfuscates the value and stores it in the cache.
///
/// # Arguments
///
/// * value - The input value to be obfuscated.
/// * delta - Sensitivity.
/// * epsilon - Privacy budget parameter.
/// * bin - The bin that the value belongs to.
/// * obf_cache_option - An option that represents the obfuscation cache.
/// * obfuscate_zero - A flag indicating whether zero counts should be obfuscated.
/// * below_10_obfuscation_mode: 0 - return 0, 1 - return 10, 2 - obfuscate using Laplace distribution and rounding
/// * rounding_step - The granularity of the rounding.
/// * rng - A secure random generator for seeded randomness.
///
/// # Returns
///
/// The obfuscated value, rounded to the nearest multiple of the rounding_step, or an error if the
/// obfuscation failed.
pub fn get_from_cache_or_privatize(
    value: u64,
    delta: f64,
    epsilon: f64,
    bin: Bin,
    obf_cache_option: Option<&mut ObfCache>,
    obfuscate_zero: bool,
    obfuscate_below_10_mode: ObfuscateBelow10Mode,
    rounding_step: usize,
    rng: &mut rand::rngs::ThreadRng,
) -> Result<u64, LaplaceError> {
    let obfuscated: u64 = match obf_cache_option {
        None => privatize(value, delta, epsilon, rounding_step, rng).unwrap(),
        Some(obf_cache) => {
            if !obfuscate_zero && value == 0 {
                return Ok(0);
            }

            if value < 10 {
                if obfuscate_below_10_mode == ObfuscateBelow10Mode::Zero {
                    return Ok(0);
                }
                if obfuscate_below_10_mode == ObfuscateBelow10Mode::Ten {
                    return Ok(10);
                }
            }

            let sensitivity: usize = delta.round() as usize;

            let obfuscated: u64 = match obf_cache.cache.get(&(sensitivity, value, bin)) {
                Some(obfuscated_reference) => *obfuscated_reference,
                None => {
                    let obfuscated_value =
                        privatize(value, delta, epsilon, rounding_step, rng).unwrap();

                    obf_cache
                        .cache
                        .insert((sensitivity, value, bin), obfuscated_value);
                    obfuscated_value
                }
            };
            obfuscated
        }
    };
    Ok(obfuscated)
}

/// Performs the actual perturbation of a value with the (epsilon, 0) laplacian
/// mechanism and rounds the result to the nearest step position.
///
/// # Arguments
///
/// * `value` - Clear value to permute.
/// * `sensitivity` - Sensitivity of query.
/// * `epsilon` - Privacy budget parameter.
/// * `rounding_step` - Rounding to the given number is performed.
/// * rng - A secure random generator for seeded randomness.
///
/// # Returns
///
/// The obfuscated value , or an error if the obfuscation failed.
pub fn privatize(
    value: u64,
    sensitivity: f64,
    epsilon: f64,
    rounding_step: usize,
    rng: &mut rand::rngs::ThreadRng,
) -> Result<u64, LaplaceError> {
    let obfuscated_value = value as f64 + laplace(0.0, sensitivity / epsilon, rng).unwrap();
    round_parametric(obfuscated_value, rounding_step)
}

/// Rounds the value to the nearest multiple of the step parameter.
///
/// # Arguments
///
/// * `value` - The value to be rounded.
/// * `stepParameter` - The step to round to, for example, 1, 5, or 10.
///
/// # Returns
///
/// Returns the rounded value, or an error if the rounding failed.
fn round_parametric(value: f64, step_parameter: usize) -> Result<u64, LaplaceError> {
    if step_parameter == 0 {
        return Err(LaplaceError::InvalidArgRoundingStepZero);
    }
    Ok((value / step_parameter as f64).round() as u64 * step_parameter as u64)
}

/// Draw a sample from a Laplace distribution.
///
/// # Arguments
///
/// * `mu` - the mean of the distribution.
/// * `b` - the scale parameter of the distribution, often equal to `sensitivity`/`epsilon`.
/// /// * `rng` - random generator.
///
/// # Returns
///
/// Returns a random sample from the Laplace distribution with the given `mu` and `b`, or an error if the distribution creation failed.
fn laplace(mu: f64, b: f64, rng: &mut rand::rngs::ThreadRng) -> Result<f64, LaplaceError> {
    let dist =
        Laplace::new(mu, b).map_err(|e| LaplaceError::DistributionCreationError(e))?;
    Ok(dist.sample(rng))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_laplace_err() {
        let mu = 10.0;
        let b = 0.0;
        let mut rng = rand::thread_rng();
        let result = laplace(mu, b, &mut rng);
        assert!(result.is_err());
    }

    #[test]
    fn test_laplace_ok() {
        let mu = 10.0;
        let b = 1.0;
        let mut rng = rand::thread_rng();
        let result = laplace(mu, b, &mut rng);
        assert!(result.is_ok());
    }

    #[test]
    fn test_laplace_output_within_range() {
        let mu = 10.0;
        let b = 1.0;
        let mut rng = rand::thread_rng();
        let result = laplace(mu, b, &mut rng).unwrap();
        assert!(result >= mu - 10.0 * b && result <= mu + 10.0 * b);
    }

    #[test]
    fn test_round_parametric() {
        assert_eq!(round_parametric(3.2, 1).unwrap(), 3);
        assert_eq!(round_parametric(3.7, 1).unwrap(), 4);
        assert_eq!(round_parametric(12.8, 5).unwrap(), 15);
        assert_eq!(round_parametric(17.4, 5).unwrap(), 15);
        assert_eq!(round_parametric(38.2, 10).unwrap(), 40);
        assert_eq!(round_parametric(44.9, 10).unwrap(), 40);
    }

    #[test]
    fn test_round_parametric_zero() {
        assert_eq!(round_parametric(0.0, 1).unwrap(), 0);
        assert_eq!(round_parametric(0.0, 5).unwrap(), 0);
        assert_eq!(round_parametric(0.0, 10).unwrap(), 0);
    }

    #[test]
    fn test_round_parametric_large() {
        assert_eq!(round_parametric(1_000_000.0, 1).unwrap(), 1_000_000);
        assert_eq!(round_parametric(1_000_000.0, 5).unwrap(), 1_000_000);
        assert_eq!(round_parametric(1_000_000.0, 10).unwrap(), 1_000_000);
    }

    #[test]
    fn test_round_parametric_invalid_step() {
        let result = round_parametric(10.0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_privatize_ok() {
        let mut rng = rand::thread_rng();
        let value = 27;
        let sensitivity = 10.0;
        let epsilon = 0.5;
        let rounding_step = 10;
        let result = privatize(
            value,
            sensitivity,
            epsilon,
            rounding_step,
            &mut rng,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_obfuscate_value_zero() {
        let mut rng = rand::thread_rng();
        let result = get_from_cache_or_privatize(0, 1.0, 1.0, 1, None, true, ObfuscateBelow10Mode::Obfuscate, 1, &mut rng);

        assert!(result.is_ok());
    }

    #[test]
    fn test_obfuscate_value_non_zero() {
        let mut rng = rand::thread_rng();
        let result = get_from_cache_or_privatize(10, 1.0, 1.0, 1, None, true, ObfuscateBelow10Mode::Obfuscate, 1, &mut rng);

        assert!(result.is_ok());
    }

    #[test]
    fn test_with_obf_cache() {
        let mut rng = rand::thread_rng();
        let mut obf_cache: ObfCache = ObfCache {
            cache: HashMap::new(),
        };

        let result =
            get_from_cache_or_privatize(10, 1.0, 1.0, 1, Some(&mut obf_cache), true, ObfuscateBelow10Mode::Obfuscate, 1, &mut rng);
        assert!(result.is_ok());

        let obfuscated_value = obf_cache.cache.get(&(1, 10, 1));
        assert!(obfuscated_value.is_some());
        let result_ok = result.unwrap();
        assert_eq!(result_ok.clone(), *obfuscated_value.unwrap());

        let result2 =
            get_from_cache_or_privatize(10, 1.0, 1.0, 1, Some(&mut obf_cache), true, ObfuscateBelow10Mode::Obfuscate, 1, &mut rng);
        assert!(result2.is_ok());
        assert_eq!(result_ok, result2.unwrap());
    }
}
