mod config;
mod db;
mod indexer;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

// ============ СТРУКТУРЫ ============

#[derive(Clone)]
struct AppState {
    db: Arc<db::Db>,
}

#[derive(Deserialize)]
struct WithdrawRequest {
    witness: String,
}

#[derive(Serialize, Deserialize)]
struct WithdrawResponse {
    proof: String,
    public_witness: String,
}

#[derive(Deserialize)]
struct CommitmentsQuery {
    pool_address: String,
}

#[derive(Serialize)]
struct CommitmentEntry {
    leaf_index: i64,
    commitment: String,
}

#[derive(Serialize)]
struct CommitmentsResponse {
    commitments: Vec<CommitmentEntry>,
}

#[derive(Serialize)]
struct RootResponse {
    root: String,
}

// ============ ХЭНДЛЕРЫ ============

async fn handle_health(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db_status = match sqlx::query("SELECT 1").execute(state.db.pool()).await {
        Ok(_) => "ok",
        Err(_) => "error",
    };

    Json(serde_json::json!({
        "status": "ok",
        "db": db_status,
    }))
}

async fn handle_withdraw(
    Json(payload): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, (StatusCode, String)> {
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

async fn handle_commitments(
    State(state): State<AppState>,
    Query(query): Query<CommitmentsQuery>,
) -> Result<Json<CommitmentsResponse>, (StatusCode, String)> {
    let rows = state
        .db
        .get_commitments(&query.pool_address)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB error: {}", e),
            )
        })?;

    let commitments = rows
        .into_iter()
        .map(|(leaf_index, commitment)| CommitmentEntry {
            leaf_index,
            commitment: hex::encode(commitment),
        })
        .collect();

    Ok(Json(CommitmentsResponse { commitments }))
}

async fn handle_root(
    State(state): State<AppState>,
    Query(query): Query<CommitmentsQuery>,
) -> Result<Json<RootResponse>, (StatusCode, String)> {
    let root = state
        .db
        .get_latest_root(&query.pool_address)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB error: {}", e),
            )
        })?;

    match root {
        Some(r) => Ok(Json(RootResponse {
            root: hex::encode(r),
        })),
        None => Err((StatusCode::NOT_FOUND, "No root found".to_string())),
    }
}

// ============ MAIN ============

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://ptrans:ptrans_dev_password@localhost:5432/ptrans".to_string()
    });
    let db = db::Db::new(&database_url).await?;
    println!("✅ Connected to Postgres");

    let db_arc = Arc::new(db);
    let state = AppState { db: db_arc.clone() };

    // Запускаем индексер в фоне
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let pool_address = std::env::var("POOL_ADDRESS")
        .unwrap_or_else(|_| "3ENojXMjs7H87eNfHMgSbbRf486rCwcAvk8sPszL9cWq".to_string());

    let indexer = indexer::Indexer::new(db_arc.clone(), rpc_url, pool_address);
    tokio::spawn(async move {
        indexer.run().await;
    });

    let app = Router::new()
        .route("/api/health", get(handle_health))
        .route("/api/withdraw", post(handle_withdraw))
        .route("/api/commitments", get(handle_commitments))
        .route("/api/root", get(handle_root))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "4001".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("🚀 Backend server running on http://{}", addr);
    println!("📡 Endpoints:");
    println!("  POST /api/withdraw    - Proxy to prover service");
    println!("  GET  /api/commitments - List all commitments");
    println!("  GET  /api/root        - Get latest root");
    println!("  GET  /api/health      - Health check (with DB status)");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
