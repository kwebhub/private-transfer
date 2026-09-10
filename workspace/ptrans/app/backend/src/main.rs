mod config;
mod db;
mod merkle_tree;

use axum::{
    extract::State,
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
    pool: Arc<merkle_tree::Pool>,
}

#[derive(Deserialize)]
struct DepositRequest {
    commitment: String, // hex
    amount: u64,
}

#[derive(Serialize)]
struct DepositResponse {
    leaf_index: u64,
    new_root: String,
    success: bool,
}

#[derive(Deserialize)]
struct ProofRequest {
    leaf_index: u64,
}

#[derive(Serialize)]
struct ProofResponse {
    merkle_proof: Vec<String>,
    is_even: Vec<bool>,
    root: String,
}

#[derive(Serialize)]
struct RootResponse {
    root: String,
    total_deposits: u64,
    capacity: u64,
}

// ============ ХЭНДЛЕРЫ ============

async fn handle_deposit(
    State(state): State<AppState>,
    Json(payload): Json<DepositRequest>,
) -> Result<Json<DepositResponse>, (StatusCode, String)> {
    let commitment_bytes = hex::decode(&payload.commitment).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid hex commitment".to_string(),
        )
    })?;

    if commitment_bytes.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Commitment must be 32 bytes".to_string(),
        ));
    }

    let mut commitment_array = [0u8; 32];
    commitment_array.copy_from_slice(&commitment_bytes);

    // Добавляем commitment в дерево
    let (leaf_index, new_root) = state
        .pool
        .add_commitment(commitment_array)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Сохраняем в БД
    // ...

    Ok(Json(DepositResponse {
        leaf_index,
        new_root: hex::encode(new_root),
        success: true,
    }))
}

async fn handle_proof(
    State(state): State<AppState>,
    Json(payload): Json<ProofRequest>,
) -> Result<Json<ProofResponse>, (StatusCode, String)> {
    let (proof, is_even, root) = state
        .pool
        .get_merkle_proof(payload.leaf_index)
        .await
        .ok_or((StatusCode::NOT_FOUND, "Leaf index not found".to_string()))?;

    Ok(Json(ProofResponse {
        merkle_proof: proof.iter().map(hex::encode).collect(),
        is_even,
        root: hex::encode(root),
    }))
}

async fn handle_root(State(state): State<AppState>) -> Json<RootResponse> {
    let (root, count) = state.pool.get_root().await;
    Json(RootResponse {
        root: hex::encode(root),
        total_deposits: count,
        capacity: 1 << merkle_tree::TREE_DEPTH,
    })
}

async fn handle_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

// ============ MAIN ============

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Инициализация пула
    let pool = merkle_tree::Pool::new().await?;
    let state = AppState {
        pool: Arc::new(pool),
    };

    let app = Router::new()
        .route("/api/deposit", post(handle_deposit))
        .route("/api/proof", post(handle_proof))
        .route("/api/root", get(handle_root))
        .route("/api/health", get(handle_health))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "4001".to_string());
    let addr = format!("0.0.0.0:{}", port).parse()?;

    println!("🚀 Backend server running on http://{}", addr);
    println!("📡 Endpoints:");
    println!("  POST /api/deposit - Add deposit to Merkle tree");
    println!("  POST /api/proof   - Get Merkle proof for leaf");
    println!("  GET  /api/root    - Get current Merkle root");
    println!("  GET  /api/health  - Health check");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
