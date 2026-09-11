use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

// ============ СТРУКТУРЫ ============

#[derive(Clone)]
struct AppState {}

#[derive(Deserialize)]
struct WithdrawRequest {
    witness: String, // base64
}

#[derive(Serialize, Deserialize)]
struct WithdrawResponse {
    proof: String,          // base64 — 324 байта
    public_witness: String, // base64 — 140 байт
}

// ============ ХЭНДЛЕРЫ ============

async fn handle_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn handle_withdraw(
    Json(payload): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, (StatusCode, String)> {
    // Проксируем запрос в prover-сервис
    let prover_url =
        std::env::var("PROVER_URL").unwrap_or_else(|_| "http://localhost:4002".to_string());
    let url = format!("{}/prove", prover_url);

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "witness": payload.witness }))
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("Prover request failed: {}", e),
            )
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("Prover error {}: {}", status, body),
        ));
    }

    let prover_response: WithdrawResponse = resp.json().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Invalid prover response: {}", e),
        )
    })?;

    Ok(Json(prover_response))
}

// ============ MAIN ============

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let state = AppState {};

    let app = Router::new()
        .route("/api/health", get(handle_health))
        .route("/api/withdraw", post(handle_withdraw))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "4001".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("🚀 Backend server running on http://{}", addr);
    println!("📡 Endpoints:");
    println!("  POST /api/withdraw - Proxy to prover service");
    println!("  GET  /api/health   - Health check");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
