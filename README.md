# Samply.Laplace

Samply.Laplace is a Rust library to obfuscate discrete values using differential privacy-inspired methods.
The values are obfuscated by perturbing them with random values sampled from a laplace distribution with configurable parameters for location, sensitivity, and privacy budget. In order to not leak more information for repeated identical or equivalent queries, the perturbation values can be cached. The library exposes an API to finely control the caching behaviour, e.g. to obfuscate data that is stratified in a number of ways. Furthermore, a rounding step can be configured to never leak individual-level data.  

Optionally, true zero values can be returned unperturbed. While lowering the privacy level slightly, this can vastly improve subsequent processes for data access control.

If the optional parameter limiting the domain of the Laplace distribution is used, restricting the perturbation to the interval `[-domain_limit, domain_limit]`, the security properties change: the mechanism is no longer $`(\epsilon,0)`$-differentially private but instead only fulfills $`(\epsilon,\delta)`$-differential privacy. The parameter can be calculated as $`\delta(\epsilon,\Delta)=\sup_S\left(Pr[Z\in S]-e^\epsilon Pr[Z\in S-\Delta]\right)_+`$.

## Dependencies

The dependencies Samply.Laplace Rust library requires are:
- thiserror v2.0.3
- statrs v0.18.0
- rand v0.8.5
- anyhow v1.0.69

## Getting Started

In this section the "installation" and usage of Samply.Laplace is described.

### Include in Cargo.toml

To use Samply.Laplace in your project, please include the following dependency in your `Cargo.toml`:

```
laplace_rs = {version = "0.2.0", git = "https://github.com/samply/laplace-rs.git", branch = "main"}
```

### Example Usage

Using Samply.Laplace library:

```rust
use laplace_rs::{ObfCache, get_from_cache_or_privatize, Bin, ObfuscateBelow10Mode};

const DELTA: f64 = 1.;
const EPSILON: f64 = 0.1;
const MU: f64 = 0.;
const ROUNDING_STEP: usize = 10;
const domain_limit = None;


fn obfuscate -> Result<u64, LaplaceError> {

	let mut obf_cache: ObfCache = ObfCache { cache: HashMap::new() };
        let mut rng = thread_rng();
	
	let value = 15;
	let obfuscated = get_from_cache_or_privatize(
	    value, // The input value to be obfuscated.
	    DELTA, // Sensitivity.
	    EPSILON, // Privacy budget parameter.
	    1, // The bin that the value belongs to.
	    Some(&mut obf_cache), // An option that represents the obfuscation cache.
	    false, // A flag indicating whether zero counts should be obfuscated.
	    ObfuscateBelow10Mode::Ten, // 0 - return 0, 1 - return 10, 2 - obfuscate using Laplace distribution and rounding
	    ROUNDING_STEP, // The granularity of the rounding.
			domain_limit, // Optional limitation to the domain of the Laplace distributions
	    &mut rng, // A secure random generator for seeded randomness.
	)?;
	
	Ok(obfuscated)

}
```

## License

Distributed under the Apache-2.0 License. See [LICENSE](LICENSE) for more 


