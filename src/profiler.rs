use crate::engine::{Expr, SystemState};

/// The Lyapunov exponent used for chaos growth calculations.
/// Must match the constant in `ChaoticVar::propagate`.
const LYAPUNOV_EXPONENT: f64 = 0.1;

/// Profiles an AST for chaotic explosions BEFORE evaluation.
///
/// Recursively walks the expression tree, computing the theoretical variance
/// that would result from each `Propagate` operation. If any propagation
/// would cause the variance to exceed `max_allowed_variance`, returns an error
/// with details about the dangerous operation.
///
/// This is the "Butterfly Profiler" - it catches chaos before it happens.
///
/// # Arguments
/// * `expr` - The expression to profile
/// * `state` - The current system state (for looking up variable values)
/// * `max_allowed_variance` - The safety threshold for variance
///
/// # Returns
/// - `Ok(())` if the expression is safe to evaluate
/// - `Err(String)` with a detailed error message if a chaotic explosion is detected
pub fn profile_chaos(expr: &Expr, state: &SystemState, max_allowed_variance: f64) -> Result<(), String> {
    profile_chaos_inner(expr, state, max_allowed_variance)
}

/// Internal recursive profiling function that tracks variable definitions.
fn profile_chaos_inner(expr: &Expr, state: &SystemState, max_allowed_variance: f64) -> Result<(), String> {
    match expr {
        Expr::Var(_name) => {
            // Variable lookup: we don't check existence during profiling
            // The VM will handle undefined variable errors at runtime
            // For profiling, we assume the variable exists and has some variance
            Ok(())
        }

        Expr::Literal(_) => {
            // Literals are always safe (zero variance)
            Ok(())
        }

        Expr::Add(left, right) => {
            // Profile both sides
            profile_chaos_inner(left, state, max_allowed_variance)?;
            profile_chaos_inner(right, state, max_allowed_variance)?;

            // Compute the resulting variance if both are variables
            let left_var = eval_to_variance(left, state)?;
            let right_var = eval_to_variance(right, state)?;

            // Get covariance if both are variable references
            let cov = get_covariance_from_exprs(left, right, state);
            let combined_variance = left_var + right_var + 2.0 * cov;

            if combined_variance > max_allowed_variance {
                return Err(format!(
                    "BUTTERFLY ANOMALY: Addition of '{}' and '{}' produces variance {}, exceeding safe limit of {}",
                    expr_name(left), expr_name(right), combined_variance, max_allowed_variance
                ));
            }

            Ok(())
        }

        Expr::Propagate(inner, time_step) => {
            // First, profile the inner expression
            profile_chaos_inner(inner, state, max_allowed_variance)?;

            // Compute what the variance would be after propagation
            let pre_variance = eval_to_variance(inner, state)?;

            // Apply the Lyapunov exponential growth: Var(t) = Var(0) * exp(2 * λ * t)
            let post_variance = pre_variance * (2.0 * LYAPUNOV_EXPONENT * time_step).exp();

            if post_variance > max_allowed_variance {
                return Err(format!(
                    "BUTTERFLY ANOMALY: Propagating '{}' for {}s expands variance from {} to {}, exceeding safe limit of {}",
                    expr_name(inner), time_step, pre_variance, post_variance, max_allowed_variance
                ));
            }

            Ok(())
        }

        Expr::Collapse(inner) => {
            // Collapse produces a scalar (zero variance), always safe
            // But still profile the inner expression for consistency
            profile_chaos_inner(inner, state, max_allowed_variance)
        }

        Expr::ChaoticConstructor(_, _) => {
            // Chaotic constructor creates a new variable with known variance, always safe
            Ok(())
        }

        Expr::Let(name, expr) => {
            // Profile the expression being assigned
            profile_chaos_inner(expr, state, max_allowed_variance)?;
            // Note: We don't track the variable in state for profiling purposes
            // The VM will handle the actual variable creation
            // For profiling, we just need to ensure the expression is safe
            let _ = name;
            Ok(())
        }

        Expr::Block(exprs) => {
            // Profile each expression
            for expr in exprs {
                profile_chaos_inner(expr, state, max_allowed_variance)?;
            }
            Ok(())
        }
    }
}

/// Evaluates an expression to extract its variance (without full evaluation).
///
/// For variables, looks up the variance in the system state.
/// For literals, returns 0.0.
/// For additions, combines variances with covariance.
/// For propagations, computes the propagated variance.
/// For collapses, returns 0.0.
fn eval_to_variance(expr: &Expr, state: &SystemState) -> Result<f64, String> {
    match expr {
        Expr::Var(name) => {
            // Look up variable variance from state; default to 0.0 if not found
            // (variables defined by `let` may not be in state during profiling)
            Ok(state.get_var(name).map_or(0.0, |v| v.variance))
        }

        Expr::Literal(_) => Ok(0.0),

        Expr::Add(left, right) => {
            let left_var = eval_to_variance(left, state)?;
            let right_var = eval_to_variance(right, state)?;
            let cov = get_covariance_from_exprs(left, right, state);
            Ok(left_var + right_var + 2.0 * cov)
        }

        Expr::Propagate(inner, time_step) => {
            let inner_var = eval_to_variance(inner, state)?;
            Ok(inner_var * (2.0 * LYAPUNOV_EXPONENT * time_step).exp())
        }

        Expr::Collapse(_) => Ok(0.0), // Collapse produces a scalar

        Expr::ChaoticConstructor(_, variance) => Ok(*variance),

        Expr::Let(_, expr) => eval_to_variance(expr, state),

        Expr::Block(exprs) => {
            // Return variance of last expression
            exprs.last()
                .map(|e| eval_to_variance(e, state))
                .unwrap_or(Ok(0.0))
        }
    }
}

/// Extracts covariance between two expressions from the system state.
/// If both are variable references, looks up their covariance.
/// Otherwise, returns 0.0 (assumes independence).
fn get_covariance_from_exprs(left: &Expr, right: &Expr, state: &SystemState) -> f64 {
    match (left, right) {
        (Expr::Var(a_name), Expr::Var(b_name)) => state.covariance(a_name, b_name),
        _ => 0.0,
    }
}

/// Returns a human-readable name for an expression (for error messages).
fn expr_name(expr: &Expr) -> String {
    match expr {
        Expr::Var(name) => name.clone(),
        Expr::Literal(value) => value.to_string(),
        Expr::Add(left, right) => format!("{} + {}", expr_name(left), expr_name(right)),
        Expr::Propagate(inner, time_step) => format!("propagate({}, {})", expr_name(inner), time_step),
        Expr::Collapse(inner) => format!("collapse({})", expr_name(inner)),
        Expr::ChaoticConstructor(mean, variance) => format!("chaotic({}, {})", mean, variance),
        Expr::Let(name, _) => name.clone(),
        Expr::Block(exprs) => {
            if let Some(last) = exprs.last() {
                expr_name(last)
            } else {
                "block".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lorenz::ChaoticVar;

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
        state.add_var(
            "pressure".to_string(),
            ChaoticVar::new(101.0, 0.1, None),
        );
        state.set_covariance("temperature", "humidity", -2.0);
        state
    }

    #[test]
    fn test_profile_literal_is_safe() {
        let state = SystemState::new();
        let expr = Expr::lit(42.0);
        assert!(profile_chaos(&expr, &state, 1.0).is_ok());
    }

    #[test]
    fn test_profile_variable_is_safe() {
        let state = make_test_state();
        let expr = Expr::var("temperature");
        assert!(profile_chaos(&expr, &state, 1.0).is_ok());
    }

    #[test]
    fn test_profile_variable_not_found() {
        // Note: The profiler no longer checks for undefined variables
        // The VM will handle undefined variable errors at runtime
        // This test now verifies that profiling succeeds for unknown variables
        let state = SystemState::new();
        let expr = Expr::var("nonexistent");
        let result = profile_chaos(&expr, &state, 1.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_profile_add_within_limit() {
        let state = make_test_state();
        let expr = Expr::add(Expr::var("temperature"), Expr::var("humidity"));
        // Variance = 0.25 + 25 + 2*(-2.0) = 21.25
        assert!(profile_chaos(&expr, &state, 100.0).is_ok());
    }

    #[test]
    fn test_profile_add_exceeds_limit() {
        let state = make_test_state();
        let expr = Expr::add(Expr::var("temperature"), Expr::var("humidity"));
        // Variance = 21.25, should fail with limit of 10.0
        let result = profile_chaos(&expr, &state, 10.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("BUTTERFLY ANOMALY"));
    }

    #[test]
    fn test_profile_propagate_within_limit() {
        let state = make_test_state();
        // pressure has variance 0.1, propagate for 1.0s
        // Post-variance = 0.1 * exp(2 * 0.1 * 1.0) = 0.1 * exp(0.2) ≈ 0.122
        let expr = Expr::propagate(Expr::var("pressure"), 1.0);
        assert!(profile_chaos(&expr, &state, 1.0).is_ok());
    }

    #[test]
    fn test_profile_propagate_exceeds_limit() {
        let state = make_test_state();
        // pressure + pressure = variance 0.2
        // propagate for 50.0s => 0.2 * exp(2 * 0.1 * 50) = 0.2 * exp(10) ≈ 4405
        let expr = Expr::propagate(
            Expr::add(Expr::var("pressure"), Expr::var("pressure")),
            50.0,
        );
        let result = profile_chaos(&expr, &state, 100.0);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("BUTTERFLY ANOMALY"));
        assert!(err_msg.contains("50s"));
    }

    #[test]
    fn test_profile_propagate_just_under_limit() {
        let state = make_test_state();
        // temperature variance = 0.25
        // propagate for 5.0s => 0.25 * exp(2 * 0.1 * 5) = 0.25 * exp(1.0) ≈ 0.68
        let expr = Expr::propagate(Expr::var("temperature"), 5.0);
        assert!(profile_chaos(&expr, &state, 1.0).is_ok());
    }

    #[test]
    fn test_profile_propagate_just_over_limit() {
        let state = make_test_state();
        // temperature variance = 0.25
        // propagate for 10.0s => 0.25 * exp(2 * 0.1 * 10) = 0.25 * exp(2.0) ≈ 1.85
        let expr = Expr::propagate(Expr::var("temperature"), 10.0);
        let result = profile_chaos(&expr, &state, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_profile_collapse_is_safe() {
        let state = make_test_state();
        let expr = Expr::collapse(Expr::var("temperature"));
        assert!(profile_chaos(&expr, &state, 0.1).is_ok()); // Even tiny limit is fine
    }

    #[test]
    fn test_profile_complex_expression_safe() {
        let state = make_test_state();
        // propagate(temp + humidity, 1.0) => variance ≈ (21.25) * exp(0.2) ≈ 25.95
        let expr = Expr::propagate(
            Expr::add(Expr::var("temperature"), Expr::var("humidity")),
            1.0,
        );
        assert!(profile_chaos(&expr, &state, 100.0).is_ok());
    }

    #[test]
    fn test_profile_complex_expression_dangerous() {
        let state = make_test_state();
        // propagate(temp + humidity, 5.0) => variance ≈ (21.25) * exp(1.0) ≈ 57.8
        let expr = Expr::propagate(
            Expr::add(Expr::var("temperature"), Expr::var("humidity")),
            5.0,
        );
        let result = profile_chaos(&expr, &state, 50.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_profile_nested_propagate() {
        let state = make_test_state();
        // propagate(propagate(temp, 1.0), 1.0)
        // Inner: 0.25 * exp(0.2) ≈ 0.303
        // Outer: 0.303 * exp(0.2) ≈ 0.370
        let expr = Expr::propagate(
            Expr::propagate(Expr::var("temperature"), 1.0),
            1.0,
        );
        assert!(profile_chaos(&expr, &state, 1.0).is_ok());
    }

    #[test]
    fn test_profile_error_message_contains_details() {
        let state = make_test_state();
        let expr = Expr::propagate(Expr::var("pressure"), 100.0);
        let result = profile_chaos(&expr, &state, 1.0);
        let err = result.unwrap_err();
        // Should contain the variable name, time step, and limit
        assert!(err.contains("pressure"));
        assert!(err.contains("100s"));
        assert!(err.contains("1"));
    }
}