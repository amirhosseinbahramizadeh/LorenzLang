use wasm_bindgen::prelude::*;
use serde::Serialize;

use crate::engine::SystemState;
use crate::parser::parse;
use crate::profiler::profile_chaos;
use crate::compiler::Compiler;
use crate::vm::ChaosVM;

const MAX_ALLOWED_VARIANCE: f64 = 1000.0;

#[derive(Serialize)]
pub struct LorenzResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub debug_state: Option<String>,
}

#[wasm_bindgen]
pub fn execute_lorenz(source_code: &str) -> JsValue {
    let result = execute_lorenz_inner(source_code);
    serde_wasm_bindgen::to_value(&result).unwrap_or_else(|_| {
        JsValue::from_str(&serde_json::to_string(&result).unwrap_or_default())
    })
}

fn execute_lorenz_inner(source_code: &str) -> LorenzResult {
    // Step 1: Parse
    let ast = match parse(source_code) {
        Ok(ast) => ast,
        Err(e) => {
            return LorenzResult {
                success: false,
                output: None,
                error: Some(format!("Parse Error: {}", e)),
                debug_state: None,
            };
        }
    };

    // Step 2: Create empty system state
    let state = SystemState::new();

    // Step 3: Profile for chaotic explosions
    if let Err(msg) = profile_chaos(&ast, &state, MAX_ALLOWED_VARIANCE) {
        return LorenzResult {
            success: false,
            output: None,
            error: Some(msg),
            debug_state: None,
        };
    }

    // Step 4: Compile to bytecode
    let bytecode = match Compiler::compile(&ast) {
        Ok(bytecode) => bytecode,
        Err(e) => {
            return LorenzResult {
                success: false,
                output: None,
                error: Some(format!("Compilation Error: {}", e)),
                debug_state: None,
            };
        }
    };

    // Step 5: Execute on ChaosVM (silent mode for WASM)
    let mut vm = ChaosVM::new(bytecode, state);
    match vm.run_silent() {
        Ok((output, debug_state)) => LorenzResult {
            success: true,
            output: Some(format!("{:.6}", output)),
            error: None,
            debug_state: Some(debug_state),
        },
        Err(e) => LorenzResult {
            success: false,
            output: None,
            error: Some(format!("VM Error: {}", e)),
            debug_state: None,
        },
    }
}

// Fallback serialization using serde_json if serde_wasm_bindgen isn't available
mod serde_wasm_bindgen {
    use super::*;
    use serde::Serialize;

    pub fn to_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
        let json_str = serde_json::to_string(value)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
        Ok(JsValue::from_str(&json_str))
    }
}