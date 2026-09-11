use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;
use tower_http::cors::CorsLayer;

const CIRCUIT_PATH: &str = "/home/ubuntu/workspace/ptrans/app/circuits/withdrawal";
const CIRCUIT_JSON: &str = "target/withdrawal.json";
const CIRCUIT_CCS: &str = "target/withdrawal.ccs";
const CIRCUIT_PK: &str = "target/withdrawal.pk";
const CIRCUIT_PROOF: &str = "target/withdrawal.proof";
const CIRCUIT_PW: &str = "target/withdrawal.pw";

#[derive(Deserialize)]
struct ProveRequest {
    witness: String, // base64
}

#[derive(Serialize)]
struct ProveResponse {
    proof: String,
    public_witness: String,
}

async fn handle_prove(
    Json(payload): Json<ProveRequest>,
) -> Result<Json<ProveResponse>, (StatusCode, String)> {
    let witness_bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &payload.witness)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64: {}", e)))?;

    let tmp_dir = std::env::temp_dir();
    let witness_path = tmp_dir.join(format!("witness-{}.gz", uuid::Uuid::new_v4()));
    tokio::fs::write(&witness_path, &witness_bytes)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Write failed: {}", e),
            )
        })?;

    let circuit_dir = PathBuf::from(CIRCUIT_PATH);
    let output = Command::new("sunspot")
        .arg("prove")
        .arg(circuit_dir.join(CIRCUIT_JSON))
        .arg(&witness_path)
        .arg(circuit_dir.join(CIRCUIT_CCS))
        .arg(circuit_dir.join(CIRCUIT_PK))
        .current_dir(&circuit_dir)
        .output()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("sunspot failed: {}", e),
            )
        })?;

    let _ = tokio::fs::remove_file(&witness_path).await;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("sunspot error: {}", stderr),
        ));
    }

    let proof_bytes = tokio::fs::read(circuit_dir.join(CIRCUIT_PROOF))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Read proof failed: {}", e),
            )
        })?;
    let pw_bytes = tokio::fs::read(circuit_dir.join(CIRCUIT_PW))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Read pw failed: {}", e),
            )
        })?;

    Ok(Json(ProveResponse {
        proof: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &proof_bytes),
        public_witness: base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &pw_bytes,
        ),
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let app = Router::new()
        .route("/prove", post(handle_prove))
        .layer(CorsLayer::permissive());

    let port = std::env::var("PROVER_PORT").unwrap_or_else(|_| "4002".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("🔐 Prover service running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
