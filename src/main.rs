mod engine;
mod parser;
mod profiler;
mod compiler;
mod vm;

use std::env;
use std::fs;
use std::process;

use engine::SystemState;
use parser::parse;
use profiler::profile_chaos;
use compiler::Compiler;
use vm::ChaosVM;

const MAX_ALLOWED_VARIANCE: f64 = 1000.0;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: lorenz <filename.lz>");
        process::exit(1);
    }

    let filename = &args[1];

    // Read the file
    let code = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Error: File '{}' not found.", filename);
            process::exit(1);
        }
    };

    // Step 1: Parse
    let ast = match parse(&code) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Lorenz Parse Error: {}", e);
            process::exit(1);
        }
    };

    // Step 2: Create system state (empty for now)
    let state = SystemState::new();

    // Step 3: Profile for chaotic explosions
    if let Err(msg) = profile_chaos(&ast, &state, MAX_ALLOWED_VARIANCE) {
        eprintln!("{}", msg);
        process::exit(1);
    }

    // Step 4: Compile to bytecode
    let bytecode = match Compiler::compile(&ast) {
        Ok(bytecode) => bytecode,
        Err(e) => {
            eprintln!("Compilation error: {}", e);
            process::exit(1);
        }
    };

    // Step 5: Execute on ChaosVM
    let mut vm = ChaosVM::new(bytecode, state);
    match vm.run() {
        Ok(result) => {
            println!("Lorenz Output: {}", result);
        }
        Err(e) => {
            eprintln!("VM error: {}", e);
            process::exit(1);
        }
    }
}