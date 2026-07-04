use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

use crate::engine::SystemState;
use crate::parser::parse;
use crate::profiler::profile_chaos;
use crate::compiler::Compiler;
use crate::vm::ChaosVM;

const MAX_ALLOWED_VARIANCE: f64 = 1000.0;

#[derive(Deserialize)]
struct EvaluateRequest {
    code: String,
}

#[derive(Serialize)]
struct EvaluateResponse {
    mean: f64,
    variance: f64,
    output: f64,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn evaluate(req: web::Json<EvaluateRequest>) -> impl Responder {
    let code = &req.code;

    // Step 1: Parse
    let ast = match parse(code) {
        Ok(ast) => ast,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Parse Error: {}", e),
            });
        }
    };

    // Step 2: Create system state
    let state = SystemState::new();

    // Step 3: Profile for chaotic explosions
    if let Err(msg) = profile_chaos(&ast, &state, MAX_ALLOWED_VARIANCE) {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: msg,
        });
    }

    // Step 4: Compile to bytecode
    let bytecode = match Compiler::compile(&ast) {
        Ok(bytecode) => bytecode,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Compilation Error: {}", e),
            });
        }
    };

    // Step 5: Execute on ChaosVM
    let mut vm = ChaosVM::new(bytecode, state);
    match vm.run() {
        Ok(output) => {
            // For now, we return the output as both mean and variance
            // A more sophisticated implementation would track the final ChaoticVar
            HttpResponse::Ok().json(EvaluateResponse {
                mean: output,
                variance: 0.0,
                output,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("VM Error: {}", e),
            })
        }
    }
}

pub async fn start_server() -> std::io::Result<()> {
    println!("Lorenz server starting on http://0.0.0.0:8080");
    println!("POST /evaluate with JSON: {{\"code\": \"your Lorenz code here\"}}");

    HttpServer::new(|| {
        App::new()
            .route("/evaluate", web::post().to(evaluate))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}