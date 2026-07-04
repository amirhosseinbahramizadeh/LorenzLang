use rand_distr::{Distribution, Normal};
use rand::rng;
use std::collections::HashMap;
use std::ops::Add;

/// Represents a variable in a chaotic system with inherent uncertainty.
///
/// Variables in chaotic systems don't hold deterministic values but instead
/// represent probability distributions that expand over time due to sensitivity
/// to initial conditions (the Butterfly Effect).
#[derive(Debug, Clone, PartialEq)]
pub struct ChaoticVar {
    /// The expected/central value of the variable (e.g., current temperature: 20.0)
    pub mean: f64,
    /// The current spread/uncertainty (variance, e.g., ±0.5 degrees → variance = 0.25)
    pub variance: f64,
    /// Sensitivity coefficients mapping other variable names to their partial derivatives.
    /// Represents how sensitive this variable is to changes in other variables
    /// (the Butterfly Effect sensitivity map).
    pub sensitivity_map: HashMap<String, f64>,
}

impl ChaoticVar {
    /// Creates a new `ChaoticVar` with the given mean and variance.
    ///
    /// # Arguments
    /// * `mean` - The expected value
    /// * `variance` - The variance (must be non-negative)
    /// * `sensitivity_map` - Optional sensitivity map (defaults to empty)
    ///
    /// # Panics
    /// Panics if `variance` is negative.
    pub fn new(mean: f64, variance: f64, sensitivity_map: Option<HashMap<String, f64>>) -> Self {
        assert!(variance >= 0.0, "variance must be non-negative");
        Self {
            mean,
            variance,
            sensitivity_map: sensitivity_map.unwrap_or_default(),
        }
    }

    /// Creates a new `ChaoticVar` with zero variance (deterministic value).
    pub fn deterministic(mean: f64) -> Self {
        Self::new(mean, 0.0, None)
    }

    /// Returns the standard deviation (square root of variance).
    pub fn std_dev(&self) -> f64 {
        self.variance.sqrt()
    }

    /// Adds a sensitivity coefficient for another variable.
    pub fn add_sensitivity(&mut self, variable: String, coefficient: f64) {
        self.sensitivity_map.insert(variable, coefficient);
    }

    /// Gets the sensitivity coefficient for a given variable.
    pub fn sensitivity(&self, variable: &str) -> Option<f64> {
        self.sensitivity_map.get(variable).copied()
    }
}

/// Trait defining chaotic operations for `ChaoticVar`.
///
/// Implements the chaotic arithmetic and time-evolution operations
/// that model the Butterfly Effect in dynamical systems.
pub trait ChaoticOps {
    /// Simulates the passage of time in a chaotic system.
    ///
    /// Uncertainty grows exponentially over time according to a simplified
    /// Lyapunov exponent model: `variance *= exp(2 * λ * dt)`
    /// where λ (lambda) = 0.1 is the Lyapunov exponent (chaos growth rate).
    ///
    /// # Arguments
    /// * `time_step` - The time step Δt to propagate forward (must be non-negative)
    ///
    /// # Panics
    /// Panics if `time_step` is negative.
    fn propagate(&mut self, time_step: f64);

    /// Collapses the probability cloud to a single concrete value.
    ///
    /// Samples from the Normal distribution N(mean, variance) using the `rand` crate.
    /// This represents "measuring" the chaotic variable, collapsing its probability cloud
    /// to a single observed value.
    ///
    /// # Returns
    /// A sampled `f64` from N(mean, √variance)
    ///
    /// # Panics
    /// Panics if variance is negative (should never happen if invariants are maintained)
    /// or if the normal distribution cannot be constructed.
    fn collapse(&self) -> f64;
}

impl ChaoticOps for ChaoticVar {
    fn propagate(&mut self, time_step: f64) {
        assert!(time_step >= 0.0, "time_step must be non-negative");

        // Simplified Lyapunov exponent (chaos growth rate)
        const LYAPUNOV_EXPONENT: f64 = 0.1;

        // Variance grows as: Var(t) = Var(0) * exp(2 * λ * t)
        // This models exponential divergence of trajectories in chaotic systems
        let growth_factor = (2.0 * LYAPUNOV_EXPONENT * time_step).exp();
        self.variance *= growth_factor;

        // Sensitivity coefficients also grow exponentially with the same Lyapunov exponent
        // This models the Butterfly Effect: sensitivity to initial conditions grows exponentially
        for coeff in self.sensitivity_map.values_mut() {
            *coeff *= (LYAPUNOV_EXPONENT * time_step).exp();
        }
    }

    fn collapse(&self) -> f64 {
        assert!(self.variance >= 0.0, "variance must be non-negative");

        let std_dev = self.variance.sqrt();

        // Handle deterministic case (zero variance)
        if std_dev == 0.0 {
            return self.mean;
        }

        let normal = Normal::new(self.mean, std_dev)
            .expect("Failed to create normal distribution; mean and std_dev must be valid");

        let mut rng = rng();
        normal.sample(&mut rng)
    }
}

impl Add<&ChaoticVar> for &ChaoticVar {
    type Output = ChaoticVar;

    fn add(self, other: &ChaoticVar) -> ChaoticVar {
        // Mean of sum = sum of means
        let new_mean = self.mean + other.mean;

        // Variance of sum = sum of variances (for independent variables)
        // Note: This assumes independence. For correlated variables, sensitivity_map would need to be merged.
        let new_variance = self.variance + other.variance;

        // Merge sensitivity maps: sum coefficients for same variables
        let mut new_sensitivity = self.sensitivity_map.clone();
        for (key, value) in &other.sensitivity_map {
            *new_sensitivity.entry(key.clone()).or_insert(0.0) += value;
        }

        ChaoticVar {
            mean: new_mean,
            variance: new_variance,
            sensitivity_map: new_sensitivity,
        }
    }
}

/// Implement `Add` for owned `ChaoticVar` as well for convenience.
impl Add<ChaoticVar> for ChaoticVar {
    type Output = ChaoticVar;

    fn add(self, other: ChaoticVar) -> ChaoticVar {
        &self + &other
    }
}

impl Add<&ChaoticVar> for ChaoticVar {
    type Output = ChaoticVar;

    fn add(self, other: &ChaoticVar) -> ChaoticVar {
        &self + other
    }
}

impl Add<ChaoticVar> for &ChaoticVar {
    type Output = ChaoticVar;

    fn add(self, other: ChaoticVar) -> ChaoticVar {
        self + &other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaotic_var_new() {
        let var = ChaoticVar::new(10.0, 4.0, None);
        assert_eq!(var.mean, 10.0);
        assert_eq!(var.variance, 4.0);
        assert_eq!(var.std_dev(), 2.0);
    }

    #[test]
    #[should_panic(expected = "variance must be non-negative")]
    fn test_chaotic_var_negative_variance_panics() {
        ChaoticVar::new(10.0, -1.0, None);
    }

    #[test]
    fn test_deterministic() {
        let var = ChaoticVar::deterministic(42.0);
        assert_eq!(var.mean, 42.0);
        assert_eq!(var.variance, 0.0);
        assert_eq!(var.collapse(), 42.0); // Deterministic collapse
    }

    #[test]
    fn test_add_means_and_variances() {
        let a = ChaoticVar::new(10.0, 4.0, None);
        let b = ChaoticVar::new(20.0, 9.0, None);
        let c = &a + &b;

        assert_eq!(c.mean, 30.0);
        assert_eq!(c.variance, 13.0); // 4 + 9
    }

    #[test]
    fn test_add_sensitivity_maps() {
        let mut a = ChaoticVar::new(10.0, 1.0, None);
        a.add_sensitivity("x".to_string(), 0.5);
        a.add_sensitivity("y".to_string(), 1.5);

        let mut b = ChaoticVar::new(20.0, 1.0, None);
        b.add_sensitivity("y".to_string(), 2.0);
        b.add_sensitivity("z".to_string(), 0.5);

        let c = &a + &b;

        assert!((c.sensitivity("x").unwrap() - 0.5).abs() < f64::EPSILON);
        assert!((c.sensitivity("y").unwrap() - 3.5).abs() < f64::EPSILON); // 1.5 + 2.0
        assert!((c.sensitivity("z").unwrap() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_propagate_increases_variance() {
        let mut var = ChaoticVar::new(10.0, 1.0, None);
        let initial_variance = var.variance;

        var.propagate(1.0);

        // Variance should grow by exp(2 * 0.1 * 1.0) = exp(0.2) ≈ 1.221
        let expected_growth = (0.2_f64).exp();
        assert!((var.variance - initial_variance * expected_growth).abs() < 1e-10);
    }

    #[test]
    fn test_propagate_increases_sensitivity() {
        let mut var = ChaoticVar::new(10.0, 1.0, None);
        var.add_sensitivity("x".to_string(), 2.0);

        let initial_sensitivity = var.sensitivity("x").unwrap();
        var.propagate(1.0);

        let expected_growth = (0.1_f64).exp(); // λ * t = 0.1 * 1.0
        assert!((var.sensitivity("x").unwrap() - initial_sensitivity * expected_growth).abs() < 1e-10);
    }

    #[test]
    #[should_panic(expected = "time_step must be non-negative")]
    fn test_propagate_negative_time_step_panics() {
        let mut var = ChaoticVar::new(10.0, 1.0, None);
        var.propagate(-1.0);
    }

    #[test]
    fn test_collapse_deterministic() {
        let var = ChaoticVar::deterministic(42.0);
        // Deterministic value should always return the mean
        for _ in 0..100 {
            assert_eq!(var.collapse(), 42.0);
        }
    }

    #[test]
    fn test_collapse_samples_from_distribution() {
        let var = ChaoticVar::new(100.0, 25.0, None); // mean=100, std=5
        let mut sum = 0.0;
        let samples = 10000;

        for _ in 0..samples {
            sum += var.collapse();
        }

        let mean = sum / samples as f64;
        // Sample mean should be close to true mean (within ~0.5 for 10k samples)
        assert!((mean - 100.0).abs() < 0.5);
    }

    #[test]
    fn test_add_owned_and_ref_combinations() {
        let a = ChaoticVar::new(1.0, 1.0, None);
        let b = ChaoticVar::new(2.0, 2.0, None);

        let r1 = &a + &b;
        let r2 = a.clone() + &b;
        let r3 = &a + b.clone();
        let r4 = a + b;

        assert_eq!(r1.mean, 3.0);
        assert_eq!(r1.variance, 3.0);
        assert_eq!(r2.mean, 3.0);
        assert_eq!(r3.mean, 3.0);
        assert_eq!(r4.mean, 3.0);
    }
}