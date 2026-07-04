use lorenz::{ChaoticVar, ChaoticOps};
use std::collections::HashMap;

/// Represents the global state of a chaotic system.
///
/// Tracks all variables and their pairwise covariances. Covariance captures
/// how two variables co-vary (e.g., when temperature rises, humidity may drop),
/// which is essential for accurately modeling the Butterfly Effect in coupled systems.
#[derive(Debug, Clone)]
pub struct SystemState {
    /// All chaotic variables in the system, keyed by name.
    pub variables: HashMap<String, ChaoticVar>,
    /// Pairwise covariance matrix between variables.
    /// Key: (variable_a, variable_b), Value: covariance coefficient.
    /// Initialized to 0.0 for all pairs (uncorrelated by default).
    pub covariances: HashMap<(String, String), f64>,
}

impl SystemState {
    /// Creates a new empty `SystemState`.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            covariances: HashMap::new(),
        }
    }

    /// Adds a variable to the system.
    pub fn add_var(&mut self, name: String, var: ChaoticVar) {
        self.variables.insert(name, var);
    }

    /// Gets a reference to a variable by name.
    pub fn get_var(&self, name: &str) -> Option<&ChaoticVar> {
        self.variables.get(name)
    }

    /// Sets the covariance between two variables.
    /// Automatically makes covariance symmetric: cov(a,b) = cov(b,a).
    #[allow(dead_code)]
    pub fn set_covariance(&mut self, a: &str, b: &str, value: f64) {
        self.covariances.insert((a.to_string(), b.to_string()), value);
        self.covariances.insert((b.to_string(), a.to_string()), value);
    }

    /// Gets the covariance between two variables.
    /// Returns 0.0 if no covariance has been set (assumes independence).
    pub fn covariance(&self, a: &str, b: &str) -> f64 {
        self.covariances.get(&(a.to_string(), b.to_string())).copied().unwrap_or(0.0)
    }

    /// Adds two variables in the system context, properly accounting for covariance.
    /// This is the covariance-aware addition that updates the global covariance matrix.
    ///
    /// # Arguments
    /// * `result_name` - Name for the resulting variable
    /// * `a_name` - Name of the first variable
    /// * `b_name` - Name of the second variable
    ///
    /// # Panics
    /// Panics if either variable is not found in the system.
    #[allow(dead_code)]
    pub fn add_vars(&mut self, result_name: &str, a_name: &str, b_name: &str) {
        let a = self.variables.get(a_name).expect("variable not found").clone();
        let b = self.variables.get(b_name).expect("variable not found").clone();
        let cov = self.covariance(a_name, b_name);

        // Mean of sum = sum of means
        let new_mean = a.mean + b.mean;

        // Variance of sum = Var(a) + Var(b) + 2*Cov(a,b)
        // This properly accounts for correlation between variables
        let new_variance = a.variance + b.variance + 2.0 * cov;

        // Merge sensitivity maps
        let mut new_sensitivity = a.sensitivity_map.clone();
        for (key, value) in &b.sensitivity_map {
            *new_sensitivity.entry(key.clone()).or_insert(0.0) += value;
        }

        let result = ChaoticVar::new(new_mean, new_variance, Some(new_sensitivity));
        self.add_var(result_name.to_string(), result);
    }

    /// Propagates all variables in the system forward in time.
    /// Also propagates covariances using the same Lyapunov exponent.
    #[allow(dead_code)]
    pub fn propagate_all(&mut self, time_step: f64) {
        // Propagate each variable
        for var in self.variables.values_mut() {
            var.propagate(time_step);
        }

        // Propagate covariances with the same exponential growth
        const LYAPUNOV_EXPONENT: f64 = 0.1;
        let growth_factor = (2.0 * LYAPUNOV_EXPONENT * time_step).exp();
        for cov in self.covariances.values_mut() {
            *cov *= growth_factor;
        }
    }

}

impl Default for SystemState {
    fn default() -> Self {
        Self::new()
    }
}

/// Abstract Syntax Tree nodes for the chaotic expression language.
///
/// Represents operations that can be performed on chaotic variables,
/// including arithmetic, time propagation, and measurement collapse.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Reference to a variable in the system by name.
    Var(String),
    /// A deterministic literal value (zero variance).
    Literal(f64),
    /// Addition of two expressions: `a + b`.
    Add(Box<Expr>, Box<Expr>),
    /// Propagate an expression forward in time by `time_step`.
    /// Returns a new ChaoticVar with increased variance.
    Propagate(Box<Expr>, f64),
    /// Collapse an expression to a deterministic value (sample from distribution).
    Collapse(Box<Expr>),
    /// A chaotic variable constructor: `chaotic(mean, variance)`.
    /// Creates a new ChaoticVar with the given mean and initial variance.
    ChaoticConstructor(f64, f64),
    /// Variable assignment: `let name = expr`.
    /// Evaluates the expression and stores the result in the system state.
    Let(String, Box<Expr>),
    /// A block of expressions evaluated in sequence. Returns the last value.
    Block(Vec<Expr>),
}

impl Expr {
    /// Helper constructor for `Expr::Var`.
    pub fn var(name: &str) -> Self {
        Expr::Var(name.to_string())
    }

    /// Helper constructor for `Expr::Literal`.
    pub fn lit(value: f64) -> Self {
        Expr::Literal(value)
    }

    /// Helper constructor for `Expr::Add`.
    pub fn add(left: Expr, right: Expr) -> Self {
        Expr::Add(Box::new(left), Box::new(right))
    }

    /// Helper constructor for `Expr::Propagate`.
    pub fn propagate(expr: Expr, time_step: f64) -> Self {
        Expr::Propagate(Box::new(expr), time_step)
    }

    /// Helper constructor for `Expr::Collapse`.
    pub fn collapse(expr: Expr) -> Self {
        Expr::Collapse(Box::new(expr))
    }

    /// Helper constructor for `Expr::ChaoticConstructor`.
    pub fn chaotic(mean: f64, variance: f64) -> Self {
        Expr::ChaoticConstructor(mean, variance)
    }

    /// Helper constructor for `Expr::Let`.
    pub fn let_binding(name: &str, expr: Expr) -> Self {
        Expr::Let(name.to_string(), Box::new(expr))
    }

    /// Helper constructor for `Expr::Block`.
    pub fn block(exprs: Vec<Expr>) -> Self {
        Expr::Block(exprs)
    }
}

/// Result of evaluating an expression in the context of a `SystemState`.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum EvalResult {
    /// A chaotic variable (probability cloud).
    Var(ChaoticVar),
    /// A deterministic scalar value (after collapse or literal).
    Scalar(f64),
}

#[allow(dead_code)]
impl EvalResult {
    /// Returns the mean of the result (whether Var or Scalar).
    pub fn mean(&self) -> f64 {
        match self {
            EvalResult::Var(v) => v.mean,
            EvalResult::Scalar(s) => *s,
        }
    }

    /// Returns the variance of the result (0.0 for Scalar).
    pub fn variance(&self) -> f64 {
        match self {
            EvalResult::Var(v) => v.variance,
            EvalResult::Scalar(_) => 0.0,
        }
    }
}

/// Evaluates an `Expr` in the context of a `SystemState`.
///
/// Returns an `EvalResult` which is either a `ChaoticVar` (probability cloud)
/// or a `Scalar` (deterministic value after collapse).
///
/// # Errors
/// Returns `EvalError` if a variable is not found in the system state.
#[allow(dead_code)]
pub fn eval(expr: &Expr, state: &SystemState) -> Result<EvalResult, EvalError> {
    match expr {
        Expr::Var(name) => {
            let var = state
                .variables
                .get(name)
                .ok_or_else(|| EvalError::VarNotFound(name.clone()))?;
            Ok(EvalResult::Var(var.clone()))
        }

        Expr::Literal(value) => Ok(EvalResult::Scalar(*value)),

        Expr::Add(left, right) => {
            let left_result = eval(left, state)?;
            let right_result = eval(right, state)?;

            match (&left_result, &right_result) {
                (EvalResult::Var(a), EvalResult::Var(b)) => {
                    // Both are chaotic variables - add with covariance awareness
                    // We need to look up covariance in the system state
                    // For now, use the names if they are Var nodes, otherwise use 0.0 covariance
                    let cov = get_covariance_from_exprs(left, right, state);
                    let new_mean = a.mean + b.mean;
                    let new_variance = a.variance + b.variance + 2.0 * cov;

                    let mut new_sensitivity = a.sensitivity_map.clone();
                    for (key, value) in &b.sensitivity_map {
                        *new_sensitivity.entry(key.clone()).or_insert(0.0) += value;
                    }

                    Ok(EvalResult::Var(ChaoticVar::new(
                        new_mean,
                        new_variance,
                        Some(new_sensitivity),
                    )))
                }
                (EvalResult::Var(a), EvalResult::Scalar(s)) => {
                    // Chaotic + deterministic = chaotic with shifted mean
                    Ok(EvalResult::Var(ChaoticVar::new(
                        a.mean + s,
                        a.variance,
                        Some(a.sensitivity_map.clone()),
                    )))
                }
                (EvalResult::Scalar(s), EvalResult::Var(b)) => {
                    // Deterministic + chaotic = chaotic with shifted mean
                    Ok(EvalResult::Var(ChaoticVar::new(
                        s + b.mean,
                        b.variance,
                        Some(b.sensitivity_map.clone()),
                    )))
                }
                (EvalResult::Scalar(a), EvalResult::Scalar(b)) => {
                    Ok(EvalResult::Scalar(a + b))
                }
            }
        }

        Expr::Propagate(inner, time_step) => {
            let result = eval(inner, state)?;
            match result {
                EvalResult::Var(mut v) => {
                    v.propagate(*time_step);
                    Ok(EvalResult::Var(v))
                }
                EvalResult::Scalar(_) => {
                    // Propagating a deterministic value doesn't change it
                    // (a constant remains constant regardless of time)
                    Ok(result)
                }
            }
        }

        Expr::Collapse(inner) => {
            let result = eval(inner, state)?;
            match result {
                EvalResult::Var(v) => Ok(EvalResult::Scalar(v.collapse())),
                EvalResult::Scalar(s) => Ok(EvalResult::Scalar(s)),
            }
        }

        Expr::ChaoticConstructor(mean, variance) => {
            let var = ChaoticVar::new(*mean, *variance, None);
            Ok(EvalResult::Var(var))
        }

        Expr::Let(_name, expr) => {
            // Note: This eval function doesn't mutate state.
            // The VM handles state mutation via the LET opcode.
            // This is provided for completeness but the CLI uses the VM.
            let result = eval(expr, state)?;
            Ok(result)
        }

        Expr::Block(exprs) => {
            let mut last_result = None;
            for expr in exprs {
                last_result = Some(eval(expr, state)?);
            }
            last_result.ok_or_else(|| EvalError::VarNotFound("empty block".to_string()))
        }
    }
}

/// Attempts to extract covariance between two expressions from the system state.
/// If both expressions are variable references, looks up their covariance.
/// Otherwise, returns 0.0 (assumes independence for complex expressions).
#[allow(dead_code)]
fn get_covariance_from_exprs(left: &Expr, right: &Expr, state: &SystemState) -> f64 {
    match (left, right) {
        (Expr::Var(a_name), Expr::Var(b_name)) => state.covariance(a_name, b_name),
        _ => 0.0, // Complex expressions: assume independence for now
    }
}

/// Errors that can occur during expression evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EvalError {
    /// A variable reference was not found in the system state.
    VarNotFound(String),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::VarNotFound(name) => write!(f, "variable '{}' not found in system state", name),
        }
    }
}

impl std::error::Error for EvalError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_state() -> SystemState {
        let mut state = SystemState::new();
        state.add_var(
            "temperature".to_string(),
            ChaoticVar::new(20.0, 0.25, None),
        );
        state.add_var(
            "humidity".to_string(),
            ChaoticVar::new(60.0, 25.0, None),
        );
        // Correlated: when temp goes up, humidity tends to go down
        state.set_covariance("temperature", "humidity", -2.0);
        state
    }

    #[test]
    fn test_system_state_new() {
        let state = SystemState::new();
        assert!(state.variables.is_empty());
        assert!(state.covariances.is_empty());
    }

    #[test]
    fn test_add_var_and_get_var() {
        let mut state = SystemState::new();
        let var = ChaoticVar::new(10.0, 1.0, None);
        state.add_var("x".to_string(), var);

        assert!(state.get_var("x").is_some());
        assert_eq!(state.get_var("x").unwrap().mean, 10.0);
        assert!(state.get_var("y").is_none());
    }

    #[test]
    fn test_covariance_symmetric() {
        let mut state = SystemState::new();
        state.set_covariance("a", "b", 5.0);

        assert_eq!(state.covariance("a", "b"), 5.0);
        assert_eq!(state.covariance("b", "a"), 5.0); // Symmetric
    }

    #[test]
    fn test_covariance_default_zero() {
        let state = SystemState::new();
        assert_eq!(state.covariance("x", "y"), 0.0);
    }

    #[test]
    fn test_add_vars_with_covariance() {
        let mut state = make_test_state();
        state.add_vars("heat_index", "temperature", "humidity");

        let result = state.get_var("heat_index").unwrap();
        // mean = 20 + 60 = 80
        assert_eq!(result.mean, 80.0);
        // variance = 0.25 + 25 + 2*(-2.0) = 21.25
        assert!((result.variance - 21.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_propagate_all() {
        let mut state = make_test_state();
        state.propagate_all(1.0);

        // Variances should have grown
        let temp = state.get_var("temperature").unwrap();
        assert!(temp.variance > 0.25);

        let humidity = state.get_var("humidity").unwrap();
        assert!(humidity.variance > 25.0);

        // Covariance should have grown too
        let cov = state.covariance("temperature", "humidity");
        assert!(cov.abs() > 2.0);
    }

    #[test]
    fn test_eval_literal() {
        let state = SystemState::new();
        let expr = Expr::lit(42.0);
        let result = eval(&expr, &state).unwrap();
        assert_eq!(result.mean(), 42.0);
        assert_eq!(result.variance(), 0.0);
    }

    #[test]
    fn test_eval_var() {
        let state = make_test_state();
        let expr = Expr::var("temperature");
        let result = eval(&expr, &state).unwrap();
        assert_eq!(result.mean(), 20.0);
    }

    #[test]
    fn test_eval_var_not_found() {
        let state = SystemState::new();
        let expr = Expr::var("nonexistent");
        let result = eval(&expr, &state);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_add_vars() {
        let state = make_test_state();
        let expr = Expr::add(Expr::var("temperature"), Expr::var("humidity"));
        let result = eval(&expr, &state).unwrap();

        assert_eq!(result.mean(), 80.0);
        // variance = 0.25 + 25 + 2*(-2.0) = 21.25
        assert!((result.variance() - 21.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_eval_propagate() {
        let state = make_test_state();
        let expr = Expr::propagate(Expr::var("temperature"), 5.0);
        let result = eval(&expr, &state).unwrap();

        // Variance should have grown
        let expected = 0.25_f64 * (2.0 * 0.1 * 5.0_f64).exp();
        assert!((result.variance() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_eval_collapse() {
        let state = make_test_state();
        let expr = Expr::collapse(Expr::var("temperature"));
        let result = eval(&expr, &state).unwrap();

        // Should be a scalar
        match result {
            EvalResult::Scalar(_) => {} // Expected
            _ => panic!("Expected Scalar result from collapse"),
        }
    }

    #[test]
    fn test_eval_complex_expression() {
        // (temperature + humidity) propagated for 2.0, then collapsed
        let state = make_test_state();
        let expr = Expr::collapse(Expr::propagate(
            Expr::add(Expr::var("temperature"), Expr::var("humidity")),
            2.0,
        ));
        let result = eval(&expr, &state).unwrap();

        match result {
            EvalResult::Scalar(v) => {
                // Should be a concrete number (varies due to randomness)
                assert!(v.is_finite());
            }
            _ => panic!("Expected Scalar from collapsed expression"),
        }
    }

    #[test]
    fn test_expr_helpers() {
        let expr = Expr::collapse(Expr::propagate(
            Expr::add(Expr::var("x"), Expr::lit(1.0)),
            0.5,
        ));

        match expr {
            Expr::Collapse(inner) => match *inner {
                Expr::Propagate(inner2, t) => {
                    assert!((t - 0.5).abs() < f64::EPSILON);
                    match *inner2 {
                        Expr::Add(left, right) => {
                            assert_eq!(*left, Expr::Var("x".to_string()));
                            assert_eq!(*right, Expr::Literal(1.0));
                        }
                        _ => panic!("Expected Add"),
                    }
                }
                _ => panic!("Expected Propagate"),
            },
            _ => panic!("Expected Collapse"),
        }
    }

    #[test]
    fn test_eval_add_scalar_and_var() {
        let state = make_test_state();
        let expr = Expr::add(Expr::lit(10.0), Expr::var("temperature"));
        let result = eval(&expr, &state).unwrap();

        assert_eq!(result.mean(), 30.0); // 10 + 20
        assert!((result.variance() - 0.25).abs() < f64::EPSILON); // Same variance as temperature
    }
}