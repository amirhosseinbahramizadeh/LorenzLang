use lorenz::{ChaoticVar, ChaoticOps};
use crate::compiler::OpCode;
use crate::engine::SystemState;
use std::collections::HashMap;

/// Stack-based Virtual Machine for executing chaotic bytecode.
///
/// The ChaosVM operates on a stack of `ChaoticVar`s, performing operations
/// that manipulate probability clouds rather than deterministic values.
/// This is what makes it fundamentally different from a standard VM - every
/// value on the stack carries uncertainty (variance) that evolves over time.
pub struct ChaosVM {
    /// The bytecode instructions to execute.
    pub code: Vec<u8>,
    /// Instruction pointer - points to the current byte being executed.
    pub ip: usize,
    /// The execution stack holding ChaoticVar values.
    pub stack: Vec<ChaoticVar>,
    /// The system state (variables and covariances).
    pub state: SystemState,
}

impl ChaosVM {
    /// Creates a new ChaosVM with the given bytecode and system state.
    pub fn new(code: Vec<u8>, state: SystemState) -> Self {
        Self {
            code,
            ip: 0,
            stack: Vec::new(),
            state,
        }
    }

    /// Runs the bytecode and returns the final collapsed f64 value.
    ///
    /// Executes instructions in a loop until a COLLAPSE opcode is encountered,
    /// which pops the top value, collapses it to a scalar, and returns it.
    pub fn run(&mut self) -> Result<f64, String> {
        loop {
            if self.ip >= self.code.len() {
                return Err("Execution ended without COLLAPSE instruction".to_string());
            }

            let opcode_byte = self.code[self.ip];
            let opcode = OpCode::from_byte(opcode_byte)
                .ok_or_else(|| format!("Unknown opcode: 0x{:02x}", opcode_byte))?;

            match opcode {
                OpCode::LOAD_VAR => {
                    self.ip += 1;
                    let name = self.read_string()?;
                    let var = self.state
                        .get_var(&name)
                        .cloned()
                        .ok_or_else(|| format!("Runtime Error: Undefined variable '{}'", name))?;
                    self.stack.push(var);
                }

                OpCode::LITERAL => {
                    self.ip += 1;
                    let value = self.read_f64()?;
                    self.stack.push(ChaoticVar::deterministic(value));
                }

                OpCode::ADD => {
                    self.ip += 1;
                    let b = self.stack_pop()?;
                    let a = self.stack_pop()?;

                    let cov = self.get_covariance_from_stack(&a, &b);
                    let new_mean = a.mean + b.mean;
                    let new_variance = a.variance + b.variance + 2.0 * cov;

                    let mut new_sensitivity = a.sensitivity_map.clone();
                    for (key, value) in &b.sensitivity_map {
                        *new_sensitivity.entry(key.clone()).or_insert(0.0) += value;
                    }

                    let result = ChaoticVar::new(new_mean, new_variance, Some(new_sensitivity));
                    self.stack.push(result);
                }

                OpCode::PROPAGATE => {
                    self.ip += 1;
                    let time_step = self.read_f64()?;
                    let mut var = self.stack_pop()?;
                    var.propagate(time_step);
                    self.stack.push(var);
                }

                OpCode::COLLAPSE => {
                    self.ip += 1;
                    let var = self.stack_pop()?;
                    eprintln!("[Lorenz State] Mean: {:.3}, Variance: {:.3}, StdDev: {:.3}", var.mean, var.variance, var.variance.sqrt());
                    return Ok(var.collapse());
                }

                OpCode::CHAOTIC => {
                    self.ip += 1;
                    let mean = self.read_f64()?;
                    let variance = self.read_f64()?;
                    let var = ChaoticVar::new(mean, variance, Some(HashMap::new()));
                    self.stack.push(var);
                }

                OpCode::LET => {
                    self.ip += 1;
                    let name = self.read_string()?;
                    let var = self.stack_pop()?;
                    self.state.add_var(name, var);
                }

                OpCode::POP => {
                    self.ip += 1;
                    self.stack_pop()?;
                }
            }
        }
    }

    /// Reads a u16 length prefix followed by that many bytes as a UTF-8 string.
    fn read_string(&mut self) -> Result<String, String> {
        if self.ip + 2 > self.code.len() {
            return Err("Truncated bytecode: missing string length".to_string());
        }

        let len = u16::from_be_bytes([self.code[self.ip], self.code[self.ip + 1]]) as usize;
        self.ip += 2;

        if self.ip + len > self.code.len() {
            return Err("Truncated bytecode: missing string data".to_string());
        }

        let bytes = &self.code[self.ip..self.ip + len];
        self.ip += len;

        String::from_utf8(bytes.to_vec())
            .map_err(|e| format!("Invalid UTF-8 in variable name: {}", e))
    }

    /// Reads an f64 from the bytecode (8 bytes, big-endian).
    fn read_f64(&mut self) -> Result<f64, String> {
        if self.ip + 8 > self.code.len() {
            return Err("Truncated bytecode: missing f64 operand".to_string());
        }

        let bytes: [u8; 8] = [
            self.code[self.ip], self.code[self.ip + 1],
            self.code[self.ip + 2], self.code[self.ip + 3],
            self.code[self.ip + 4], self.code[self.ip + 5],
            self.code[self.ip + 6], self.code[self.ip + 7],
        ];
        self.ip += 8;

        Ok(f64::from_be_bytes(bytes))
    }

    /// Pops a value from the stack, returning an error if the stack is empty.
    fn stack_pop(&mut self) -> Result<ChaoticVar, String> {
        self.stack.pop().ok_or_else(|| "Stack underflow: not enough values".to_string())
    }

    /// Attempts to extract covariance between two stack values.
    fn get_covariance_from_stack(&self, a: &ChaoticVar, b: &ChaoticVar) -> f64 {
        let common: f64 = a.sensitivity_map.iter()
            .filter_map(|(k, v1)| b.sensitivity_map.get(k).map(|v2| v1 * v2))
            .sum();
        common
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::SystemState;

    fn make_test_state() -> SystemState {
        let mut state = SystemState::new();
        state.add_var(
            "pressure".to_string(),
            ChaoticVar::new(101.0, 0.1, None),
        );
        state.add_var(
            "temperature".to_string(),
            ChaoticVar::new(20.0, 0.25, None),
        );
        state.set_covariance("pressure", "temperature", -2.0);
        state
    }

    #[test]
    fn test_vm_literal_collapse() {
        let code = vec![
            0x02, // LITERAL
            0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // f64(42.0)
            0x05, // COLLAPSE
        ];

        let mut vm = ChaosVM::new(code, SystemState::new());
        let result = vm.run().unwrap();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_vm_load_var_collapse() {
        let mut code = vec![0x01]; // LOAD_VAR
        let name = b"pressure";
        code.extend_from_slice(&(name.len() as u16).to_be_bytes());
        code.extend_from_slice(name);
        code.push(0x05); // COLLAPSE

        let mut vm = ChaosVM::new(code, make_test_state());
        let result = vm.run().unwrap();
        assert!((result - 101.0).abs() < 10.0);
    }

    #[test]
    fn test_vm_chaotic() {
        // CHAOTIC(101.0, 0.1) + COLLAPSE
        let mut code = vec![0x06]; // CHAOTIC
        code.extend_from_slice(&101.0_f64.to_be_bytes());
        code.extend_from_slice(&0.1_f64.to_be_bytes());
        code.push(0x05); // COLLAPSE

        let mut vm = ChaosVM::new(code, SystemState::new());
        let result = vm.run().unwrap();
        assert!((result - 101.0).abs() < 10.0);
    }

    #[test]
    fn test_vm_let_and_load() {
        // CHAOTIC(42.0, 0.0) + LET "x" + LOAD_VAR "x" + COLLAPSE
        let mut code = vec![0x06]; // CHAOTIC
        code.extend_from_slice(&42.0_f64.to_be_bytes());
        code.extend_from_slice(&0.0_f64.to_be_bytes());
        code.push(0x07); // LET
        let name = b"x";
        code.extend_from_slice(&(name.len() as u16).to_be_bytes());
        code.extend_from_slice(name);
        code.push(0x01); // LOAD_VAR
        code.extend_from_slice(&(name.len() as u16).to_be_bytes());
        code.extend_from_slice(name);
        code.push(0x05); // COLLAPSE

        let mut vm = ChaosVM::new(code, SystemState::new());
        let result = vm.run().unwrap();
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_vm_undefined_variable_error() {
        let mut code = vec![0x01]; // LOAD_VAR
        let name = b"nonexistent";
        code.extend_from_slice(&(name.len() as u16).to_be_bytes());
        code.extend_from_slice(name);
        code.push(0x05); // COLLAPSE

        let mut vm = ChaosVM::new(code, SystemState::new());
        let result = vm.run();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined variable"));
    }

    #[test]
    fn test_vm_add_two_chaotic() {
        // CHAOTIC(10.0, 1.0) + CHAOTIC(20.0, 2.0) + ADD + COLLAPSE
        let mut code = vec![0x06]; // CHAOTIC 10.0, 1.0
        code.extend_from_slice(&10.0_f64.to_be_bytes());
        code.extend_from_slice(&1.0_f64.to_be_bytes());
        code.push(0x06); // CHAOTIC 20.0, 2.0
        code.extend_from_slice(&20.0_f64.to_be_bytes());
        code.extend_from_slice(&2.0_f64.to_be_bytes());
        code.push(0x03); // ADD
        code.push(0x05); // COLLAPSE

        let mut vm = ChaosVM::new(code, SystemState::new());
        let result = vm.run().unwrap();
        // Mean should be close to 30.0 (10 + 20) with some variance
        assert!((result - 30.0).abs() < 10.0);
    }

    #[test]
    fn test_vm_pop() {
        // CHAOTIC(42.0, 0.0) + POP + CHAOTIC(99.0, 0.0) + COLLAPSE
        let mut code = vec![0x06]; // CHAOTIC 42.0
        code.extend_from_slice(&42.0_f64.to_be_bytes());
        code.extend_from_slice(&0.0_f64.to_be_bytes());
        code.push(0x08); // POP
        code.push(0x06); // CHAOTIC 99.0
        code.extend_from_slice(&99.0_f64.to_be_bytes());
        code.extend_from_slice(&0.0_f64.to_be_bytes());
        code.push(0x05); // COLLAPSE

        let mut vm = ChaosVM::new(code, SystemState::new());
        let result = vm.run().unwrap();
        assert_eq!(result, 99.0); // Should be 99, not 42
    }

    #[test]
    fn test_vm_complex_expression() {
        // let pressure = chaotic(101.0, 0.1)
        // let temp = chaotic(20.0, 0.5)
        // collapse(propagate(pressure + temp, 1.0))
        let mut code = Vec::new();

        // CHAOTIC(101.0, 0.1)
        code.push(0x06);
        code.extend_from_slice(&101.0_f64.to_be_bytes());
        code.extend_from_slice(&0.1_f64.to_be_bytes());

        // LET "pressure"
        code.push(0x07);
        code.extend_from_slice(b"\x00\x08");
        code.extend_from_slice(b"pressure");

        // CHAOTIC(20.0, 0.5)
        code.push(0x06);
        code.extend_from_slice(&20.0_f64.to_be_bytes());
        code.extend_from_slice(&0.5_f64.to_be_bytes());

        // LET "temp"
        code.push(0x07);
        code.extend_from_slice(b"\x00\x04");
        code.extend_from_slice(b"temp");

        // LOAD_VAR "pressure"
        code.push(0x01);
        code.extend_from_slice(b"\x00\x08");
        code.extend_from_slice(b"pressure");

        // LOAD_VAR "temp"
        code.push(0x01);
        code.extend_from_slice(b"\x00\x04");
        code.extend_from_slice(b"temp");

        // ADD
        code.push(0x03);

        // PROPAGATE 1.0
        code.push(0x04);
        code.extend_from_slice(&1.0_f64.to_be_bytes());

        // COLLAPSE
        code.push(0x05);

        let mut vm = ChaosVM::new(code, SystemState::new());
        let result = vm.run().unwrap();
        // Should be close to 121.0 (101 + 20) with some variance
        assert!((result - 121.0).abs() < 20.0);
    }

    #[test]
    fn test_vm_empty_stack_error() {
        let code = vec![0x03]; // ADD

        let mut vm = ChaosVM::new(code, SystemState::new());
        let result = vm.run();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Stack underflow"));
    }

    #[test]
    fn test_vm_unknown_opcode_error() {
        let code = vec![0xFF];

        let mut vm = ChaosVM::new(code, SystemState::new());
        let result = vm.run();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown opcode"));
    }

    #[test]
    fn test_vm_no_collapse_error() {
        let code = vec![
            0x02, // LITERAL
            0x40, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let mut vm = ChaosVM::new(code, SystemState::new());
        let result = vm.run();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("without COLLAPSE"));
    }
}