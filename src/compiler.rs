use crate::engine::Expr;

/// Bytecode opcodes for the Chaotic Virtual Machine.
///
/// Each opcode is a single byte, followed by operand bytes as needed.
/// The VM executes these instructions on a stack of `ChaoticVar`s.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum OpCode {
    /// Load a variable from the system state onto the stack.
    /// Format: `[0x01] [u16 length] [UTF-8 bytes...]`
    LOAD_VAR = 0x01,

    /// Push a literal f64 value as a ChaoticVar (mean=value, variance=0.0).
    /// Format: `[0x02] [f64 big-endian (8 bytes)]`
    LITERAL = 0x02,

    /// Pop two ChaoticVars, add them (covariance-aware), push result.
    /// Format: `[0x03]`
    ADD = 0x03,

    /// Pop a ChaoticVar, propagate it forward in time, push result.
    /// Format: `[0x04] [f64 time_step big-endian (8 bytes)]`
    PROPAGATE = 0x04,

    /// Pop a ChaoticVar, collapse it to a scalar, return as f64.
    /// This terminates execution.
    /// Format: `[0x05]`
    COLLAPSE = 0x05,

    /// Create a new ChaoticVar from mean and variance, push to stack.
    /// Format: `[0x06] [f64 mean (8 bytes)] [f64 variance (8 bytes)]`
    CHAOTIC = 0x06,

    /// Pop a ChaoticVar from stack, store in system state with given name.
    /// Format: `[0x07] [u16 length] [UTF-8 bytes...]`
    LET = 0x07,

    /// Pop and discard the top value from the stack.
    /// Format: `[0x08]`
    POP = 0x08,
}

impl OpCode {
    /// Converts a byte to an OpCode. Returns None if the byte is not a valid opcode.
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(OpCode::LOAD_VAR),
            0x02 => Some(OpCode::LITERAL),
            0x03 => Some(OpCode::ADD),
            0x04 => Some(OpCode::PROPAGATE),
            0x05 => Some(OpCode::COLLAPSE),
            0x06 => Some(OpCode::CHAOTIC),
            0x07 => Some(OpCode::LET),
            0x08 => Some(OpCode::POP),
            _ => None,
        }
    }
}

/// Bytecode compiler that transforms an `Expr` AST into a flat byte sequence.
///
/// The compiler performs a recursive post-order traversal of the AST,
/// emitting opcodes and operands in the correct order for stack-based execution.
pub struct Compiler {
    bytecode: Vec<u8>,
}

impl Compiler {
    /// Creates a new Compiler instance.
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
        }
    }

    /// Compiles an expression into bytecode.
    pub fn compile(expr: &Expr) -> Result<Vec<u8>, String> {
        let mut compiler = Compiler::new();
        compiler.compile_expr(expr)?;
        Ok(compiler.bytecode)
    }

    /// Recursively compiles an expression node.
    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Var(name) => {
                self.emit_load_var(name);
                Ok(())
            }

            Expr::Literal(value) => {
                self.emit_literal(*value);
                Ok(())
            }

            Expr::Add(left, right) => {
                // Post-order: compile left, compile right, then ADD
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                self.emit_byte(OpCode::ADD as u8);
                Ok(())
            }

            Expr::Propagate(inner, time_step) => {
                // Post-order: compile inner, then PROPAGATE with time_step
                self.compile_expr(inner)?;
                self.emit_propagate(*time_step);
                Ok(())
            }

            Expr::Collapse(inner) => {
                // Post-order: compile inner, then COLLAPSE
                self.compile_expr(inner)?;
                self.emit_byte(OpCode::COLLAPSE as u8);
                Ok(())
            }

            Expr::ChaoticConstructor(mean, variance) => {
                self.emit_chaotic(*mean, *variance);
                Ok(())
            }

            Expr::Let(name, expr) => {
                // Compile the expression, then LET to store it
                // LET pops the value from the stack and stores it
                self.compile_expr(expr)?;
                self.emit_let(name);
                Ok(())
            }

            Expr::Block(exprs) => {
                // Compile each statement
                for (i, expr) in exprs.iter().enumerate() {
                    self.compile_expr(expr)?;
                    // Pop intermediate results (except the last one)
                    // Don't pop after Let statements since LET already pops
                    if i < exprs.len() - 1 && !matches!(expr, Expr::Let(_, _)) {
                        self.emit_byte(OpCode::POP as u8);
                    }
                }
                Ok(())
            }
        }
    }

    /// Emits a single byte.
    fn emit_byte(&mut self, b: u8) {
        self.bytecode.push(b);
    }

    /// Emits a LOAD_VAR instruction with a variable name.
    fn emit_load_var(&mut self, name: &str) {
        self.bytecode.push(OpCode::LOAD_VAR as u8);
        self.emit_string(name);
    }

    /// Emits a LITERAL instruction with an f64 value.
    fn emit_literal(&mut self, value: f64) {
        self.bytecode.push(OpCode::LITERAL as u8);
        self.bytecode.extend_from_slice(&value.to_be_bytes());
    }

    /// Emits a PROPAGATE instruction with a time_step.
    fn emit_propagate(&mut self, time_step: f64) {
        self.bytecode.push(OpCode::PROPAGATE as u8);
        self.bytecode.extend_from_slice(&time_step.to_be_bytes());
    }

    /// Emits a CHAOTIC instruction with mean and variance.
    fn emit_chaotic(&mut self, mean: f64, variance: f64) {
        self.bytecode.push(OpCode::CHAOTIC as u8);
        self.bytecode.extend_from_slice(&mean.to_be_bytes());
        self.bytecode.extend_from_slice(&variance.to_be_bytes());
    }

    /// Emits a LET instruction with a variable name.
    fn emit_let(&mut self, name: &str) {
        self.bytecode.push(OpCode::LET as u8);
        self.emit_string(name);
    }

    /// Emits a string with u16 length prefix.
    fn emit_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let len = bytes.len() as u16;
        self.bytecode.extend_from_slice(&len.to_be_bytes());
        self.bytecode.extend_from_slice(bytes);
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    fn disassemble(code: &[u8]) -> String {
        let mut output = String::new();
        let mut ip = 0;
        while ip < code.len() {
            let offset = ip;
            match OpCode::from_byte(code[ip]) {
                Some(OpCode::LOAD_VAR) => {
                    ip += 1;
                    if ip + 2 > code.len() { break; }
                    let len = u16::from_be_bytes([code[ip], code[ip + 1]]) as usize;
                    ip += 2;
                    if ip + len > code.len() { break; }
                    let name = String::from_utf8_lossy(&code[ip..ip + len]);
                    output.push_str(&format!("{:04x}: LOAD_VAR \"{}\"\n", offset, name));
                    ip += len;
                }
                Some(OpCode::LITERAL) => {
                    ip += 1;
                    if ip + 8 > code.len() { break; }
                    let bytes: [u8; 8] = [code[ip], code[ip+1], code[ip+2], code[ip+3],
                                           code[ip+4], code[ip+5], code[ip+6], code[ip+7]];
                    let value = f64::from_be_bytes(bytes);
                    output.push_str(&format!("{:04x}: LITERAL {}\n", offset, value));
                    ip += 8;
                }
                Some(OpCode::ADD) => { output.push_str(&format!("{:04x}: ADD\n", offset)); ip += 1; }
                Some(OpCode::PROPAGATE) => {
                    ip += 1;
                    if ip + 8 > code.len() { break; }
                    let bytes: [u8; 8] = [code[ip], code[ip+1], code[ip+2], code[ip+3],
                                           code[ip+4], code[ip+5], code[ip+6], code[ip+7]];
                    let ts = f64::from_be_bytes(bytes);
                    output.push_str(&format!("{:04x}: PROPAGATE {}\n", offset, ts));
                    ip += 8;
                }
                Some(OpCode::COLLAPSE) => { output.push_str(&format!("{:04x}: COLLAPSE\n", offset)); ip += 1; }
                Some(OpCode::CHAOTIC) => {
                    ip += 1;
                    if ip + 16 > code.len() { break; }
                    let m: [u8; 8] = [code[ip], code[ip+1], code[ip+2], code[ip+3],
                                       code[ip+4], code[ip+5], code[ip+6], code[ip+7]];
                    let v: [u8; 8] = [code[ip+8], code[ip+9], code[ip+10], code[ip+11],
                                       code[ip+12], code[ip+13], code[ip+14], code[ip+15]];
                    output.push_str(&format!("{:04x}: CHAOTIC({}, {})\n", offset, f64::from_be_bytes(m), f64::from_be_bytes(v)));
                    ip += 16;
                }
                Some(OpCode::LET) => {
                    ip += 1;
                    if ip + 2 > code.len() { break; }
                    let len = u16::from_be_bytes([code[ip], code[ip + 1]]) as usize;
                    ip += 2;
                    if ip + len > code.len() { break; }
                    let name = String::from_utf8_lossy(&code[ip..ip + len]);
                    output.push_str(&format!("{:04x}: LET \"{}\"\n", offset, name));
                    ip += len;
                }
                Some(OpCode::POP) => { output.push_str(&format!("{:04x}: POP\n", offset)); ip += 1; }
                None => { output.push_str(&format!("{:04x}: UNKNOWN\n", offset)); ip += 1; }
            }
        }
        output
    }

    #[test]
    fn test_compile_literal() {
        let expr = Expr::lit(42.0);
        let code = Compiler::compile(&expr).unwrap();

        assert_eq!(code.len(), 9); // 1 opcode + 8 bytes f64
        assert_eq!(code[0], OpCode::LITERAL as u8);

        let bytes: [u8; 8] = [code[1], code[2], code[3], code[4],
                               code[5], code[6], code[7], code[8]];
        assert_eq!(f64::from_be_bytes(bytes), 42.0);
    }

    #[test]
    fn test_compile_variable() {
        let expr = Expr::var("temp");
        let code = Compiler::compile(&expr).unwrap();

        assert_eq!(code[0], OpCode::LOAD_VAR as u8);
        let len = u16::from_be_bytes([code[1], code[2]]);
        assert_eq!(len, 4);
        assert_eq!(&code[3..7], b"temp");
    }

    #[test]
    fn test_compile_chaotic() {
        let expr = Expr::chaotic(101.0, 0.1);
        let code = Compiler::compile(&expr).unwrap();

        assert_eq!(code[0], OpCode::CHAOTIC as u8);
        let mean_bytes: [u8; 8] = [code[1], code[2], code[3], code[4],
                                    code[5], code[6], code[7], code[8]];
        let var_bytes: [u8; 8] = [code[9], code[10], code[11], code[12],
                                   code[13], code[14], code[15], code[16]];
        assert_eq!(f64::from_be_bytes(mean_bytes), 101.0);
        assert_eq!(f64::from_be_bytes(var_bytes), 0.1);
    }

    #[test]
    fn test_compile_let() {
        let expr = Expr::let_binding("x", Expr::lit(42.0));
        let code = Compiler::compile(&expr).unwrap();

        // Should be: LITERAL 42.0 + LET "x"
        assert_eq!(code[0], OpCode::LITERAL as u8);
        let offset_let = 9; // After LITERAL + 8 bytes
        assert_eq!(code[offset_let], OpCode::LET as u8);
        let len = u16::from_be_bytes([code[offset_let + 1], code[offset_let + 2]]);
        assert_eq!(len, 1);
        assert_eq!(code[offset_let + 3], b'x');
        // No POP after LET
        assert_eq!(code.len(), offset_let + 4);
    }

    #[test]
    fn test_compile_block() {
        let expr = Expr::block(vec![
            Expr::let_binding("x", Expr::lit(1.0)),
            Expr::let_binding("y", Expr::lit(2.0)),
            Expr::add(Expr::var("x"), Expr::var("y")),
        ]);
        let code = Compiler::compile(&expr).unwrap();

        // Verify the structure manually:
        // LITERAL 1.0: [02][3FF0000000000000]
        assert_eq!(code[0], OpCode::LITERAL as u8);
        assert_eq!(code[1..9], [0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // LET "x": [07][0001][78]
        assert_eq!(code[9], OpCode::LET as u8);
        assert_eq!(code[10..12], [0x00, 0x01]); // length = 1
        assert_eq!(code[12], b'x');

        // LITERAL 2.0: [02][4000000000000000]
        assert_eq!(code[13], OpCode::LITERAL as u8);
        assert_eq!(code[14..22], [0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // LET "y": [07][0001][79]
        assert_eq!(code[22], OpCode::LET as u8);
        assert_eq!(code[23..25], [0x00, 0x01]); // length = 1
        assert_eq!(code[25], b'y');

        // LOAD_VAR "x": [01][0001][78]
        assert_eq!(code[26], OpCode::LOAD_VAR as u8);
        assert_eq!(code[27..29], [0x00, 0x01]);
        assert_eq!(code[29], b'x');

        // LOAD_VAR "y": [01][0001][79]
        assert_eq!(code[30], OpCode::LOAD_VAR as u8);
        assert_eq!(code[31..33], [0x00, 0x01]);
        assert_eq!(code[33], b'y');

        // ADD: [03]
        assert_eq!(code[34], OpCode::ADD as u8);
    }

    #[test]
    fn test_compile_add() {
        let expr = Expr::add(Expr::var("x"), Expr::var("y"));
        let code = Compiler::compile(&expr).unwrap();

        assert_eq!(code[0], OpCode::LOAD_VAR as u8);
        let len_x = u16::from_be_bytes([code[1], code[2]]) as usize;
        let offset_y = 3 + len_x;
        assert_eq!(code[offset_y], OpCode::LOAD_VAR as u8);
        let len_y = u16::from_be_bytes([code[offset_y + 1], code[offset_y + 2]]) as usize;
        let offset_add = offset_y + 3 + len_y;
        assert_eq!(code[offset_add], OpCode::ADD as u8);
    }

    #[test]
    fn test_compile_propagate() {
        let expr = Expr::propagate(Expr::var("x"), 2.5);
        let code = Compiler::compile(&expr).unwrap();

        assert_eq!(code[0], OpCode::LOAD_VAR as u8);
        let len = u16::from_be_bytes([code[1], code[2]]) as usize;
        let offset_prop = 3 + len;
        assert_eq!(code[offset_prop], OpCode::PROPAGATE as u8);

        let bytes: [u8; 8] = [code[offset_prop+1], code[offset_prop+2], code[offset_prop+3],
                               code[offset_prop+4], code[offset_prop+5], code[offset_prop+6],
                               code[offset_prop+7], code[offset_prop+8]];
        assert_eq!(f64::from_be_bytes(bytes), 2.5);
    }

    #[test]
    fn test_compile_collapse() {
        let expr = Expr::collapse(Expr::var("x"));
        let code = Compiler::compile(&expr).unwrap();

        assert_eq!(code[0], OpCode::LOAD_VAR as u8);
        let len = u16::from_be_bytes([code[1], code[2]]) as usize;
        let offset_collapse = 3 + len;
        assert_eq!(code[offset_collapse], OpCode::COLLAPSE as u8);
    }

    #[test]
    fn test_compile_complex_expression() {
        let expr = Expr::collapse(Expr::propagate(
            Expr::add(Expr::var("x"), Expr::var("y")),
            1.0,
        ));
        let code = Compiler::compile(&expr).unwrap();

        assert_eq!(code[0], OpCode::LOAD_VAR as u8);
        assert_eq!(code[1], 0);
        assert_eq!(code[2], 1);
        assert_eq!(code[3], b'x');

        assert_eq!(code[4], OpCode::LOAD_VAR as u8);
        assert_eq!(code[5], 0);
        assert_eq!(code[6], 1);
        assert_eq!(code[7], b'y');

        assert_eq!(code[8], OpCode::ADD as u8);

        assert_eq!(code[9], OpCode::PROPAGATE as u8);
        assert_eq!(code[10..18], [0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        assert_eq!(code[18], OpCode::COLLAPSE as u8);
    }

    #[test]
    fn test_disassemble_literal() {
        let expr = Expr::lit(1.0);
        let code = Compiler::compile(&expr).unwrap();
        let dis = disassemble(&code);
        assert!(dis.contains("LITERAL 1"));
    }

    #[test]
    fn test_disassemble_chaotic() {
        let expr = Expr::chaotic(101.0, 0.1);
        let code = Compiler::compile(&expr).unwrap();
        let dis = disassemble(&code);
        assert!(dis.contains("CHAOTIC(101"));
        assert!(dis.contains("0.1"));
    }

    #[test]
    fn test_disassemble_let() {
        let expr = Expr::let_binding("x", Expr::lit(42.0));
        let code = Compiler::compile(&expr).unwrap();
        let dis = disassemble(&code);
        assert!(dis.contains("LET \"x\""));
        assert!(!dis.contains("POP")); // No POP after LET
    }

    #[test]
    fn test_opcode_roundtrip() {
        assert_eq!(OpCode::from_byte(0x01), Some(OpCode::LOAD_VAR));
        assert_eq!(OpCode::from_byte(0x02), Some(OpCode::LITERAL));
        assert_eq!(OpCode::from_byte(0x03), Some(OpCode::ADD));
        assert_eq!(OpCode::from_byte(0x04), Some(OpCode::PROPAGATE));
        assert_eq!(OpCode::from_byte(0x05), Some(OpCode::COLLAPSE));
        assert_eq!(OpCode::from_byte(0x06), Some(OpCode::CHAOTIC));
        assert_eq!(OpCode::from_byte(0x07), Some(OpCode::LET));
        assert_eq!(OpCode::from_byte(0x08), Some(OpCode::POP));
        assert_eq!(OpCode::from_byte(0xFF), None);
    }

    #[test]
    fn test_long_variable_name() {
        let name = "a_very_long_variable_name_that_exceeds_255_bytes_".repeat(10);
        let expr = Expr::var(&name);
        let code = Compiler::compile(&expr).unwrap();

        let len = u16::from_be_bytes([code[1], code[2]]) as usize;
        assert_eq!(len, name.len());
        let decoded = String::from_utf8_lossy(&code[3..3 + len]);
        assert_eq!(decoded.as_ref(), name);
    }
}