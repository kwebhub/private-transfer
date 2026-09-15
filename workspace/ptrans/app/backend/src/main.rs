mod cache;
mod config;
mod db;
mod indexer;
mod rate_limit;
mod tree;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

// ============ СТРУКТУРЫ ============

#[derive(Clone)]
struct AppState {
    db: Arc<db::Db>,
    cache: Arc<cache::Cache>,
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

#[derive(Serialize, Deserialize)]
struct CommitmentEntry {
    leaf_index: i64,
    commitment: String,
}

#[derive(Serialize, Deserialize)]
struct CommitmentsResponse {
    commitments: Vec<CommitmentEntry>,
}

#[derive(Serialize, Deserialize)]
struct RootResponse {
    root: String,
}

#[derive(Deserialize)]
struct ProofQuery {
    pool_address: String,
    leaf_index: usize,
}

#[derive(Serialize, Deserialize)]
struct ProofResponse {
    proof: Vec<String>,
    is_even: Vec<bool>,
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
    let cache_key = format!("commitments:{}", query.pool_address);

    if let Ok(Some(cached)) = state.cache.get(&cache_key).await {
        if let Ok(parsed) = serde_json::from_str::<CommitmentsResponse>(&cached) {
            return Ok(Json(parsed));
        }
    }

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
        .collect::<Vec<_>>();

    let response = CommitmentsResponse { commitments };

    if let Ok(serialized) = serde_json::to_string(&response) {
        let _ = state.cache.set_ex(&cache_key, &serialized, 30).await;
    }

    Ok(Json(response))
}

async fn handle_root(
    State(state): State<AppState>,
    Query(query): Query<CommitmentsQuery>,
) -> Result<Json<RootResponse>, (StatusCode, String)> {
    if let Ok(Some(root)) = state.cache.get_tree_root(&query.pool_address).await {
        return Ok(Json(RootResponse { root }));
    }

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

async fn handle_proof(
    State(state): State<AppState>,
    Query(query): Query<ProofQuery>,
) -> Result<Json<ProofResponse>, (StatusCode, String)> {
    match tree::get_proof(&state.cache, &query.pool_address, query.leaf_index).await {
        Ok((proof, is_even, root)) => {
            if root == "0000000000000000000000000000000000000000000000000000000000000000" {
                return Err((
                    StatusCode::NOT_FOUND,
                    "Tree not initialized or empty".to_string(),
                ));
            }

            Ok(Json(ProofResponse {
                proof,
                is_even,
                root,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Tree error: {}", e),
        )),
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

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let cache = cache::Cache::new(&redis_url).await?;
    println!("✅ Connected to Redis");

    let db_arc = Arc::new(db);
    let cache_arc = Arc::new(cache);

    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
    let pool_address = std::env::var("POOL_ADDRESS")
        .unwrap_or_else(|_| "TQYnBSBF3z3FxMHupvt3cbKYXqsorYY9gYb19rjCncN".to_string());
    let merkle_url =
        std::env::var("MERKLE_URL").unwrap_or_else(|_| "http://localhost:4003".to_string());

    // Синхронизация дерева
    let tree_size = cache_arc.get_tree_size(&pool_address).await?.unwrap_or(0);
    let db_commitments = db_arc.get_commitments(&pool_address).await?;

    if tree_size == 0 && cache_arc.get_tree_root(&pool_address).await?.is_none() {
        println!("🌳 Initializing empty tree in Redis...");
        tree::init_empty_tree(&cache_arc, &merkle_url, &pool_address)
            .await
            .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
    }

    let current_size = cache_arc.get_tree_size(&pool_address).await?.unwrap_or(0);

    if db_commitments.len() > current_size {
        println!(
            "🌳 Syncing tree: DB has {} commitments, Redis has {} — adding {} more",
            db_commitments.len(),
            current_size,
            db_commitments.len() - current_size
        );

        for (leaf_index, commitment) in &db_commitments[current_size..] {
            let commitment_hex = hex::encode(commitment);
            tree::add_leaf(
                &cache_arc,
                &merkle_url,
                &pool_address,
                *leaf_index as usize,
                &commitment_hex,
            )
            .await
            .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
        }

        let final_root = cache_arc
            .get_tree_root(&pool_address)
            .await?
            .unwrap_or_else(|| "unknown".to_string());
        println!("🌳 Tree synced: root={}", final_root);
    } else {
        let root = cache_arc
            .get_tree_root(&pool_address)
            .await?
            .unwrap_or_else(|| "unknown".to_string());
        println!(
            "🌳 Tree already in sync: {} leaves, root={}",
            current_size, root
        );
    }

    let state = AppState {
        db: db_arc.clone(),
        cache: cache_arc.clone(),
    };

    let indexer = indexer::Indexer::new(
        db_arc.clone(),
        cache_arc.clone(),
        rpc_url,
        pool_address.clone(),
        merkle_url.clone(),
    );
    tokio::spawn(async move {
        indexer.run().await;
    });

    // Роуты с разными rate limit + cache в extensions
    let withdraw_routes = Router::new()
        .route("/api/withdraw", post(handle_withdraw))
        .layer(middleware::from_fn(rate_limit::rate_limit_withdraw))
        .layer(axum::Extension(cache_arc.clone()))
        .with_state(state.clone());

    let read_routes = Router::new()
        .route("/api/commitments", get(handle_commitments))
        .route("/api/root", get(handle_root))
        .route("/api/proof", get(handle_proof))
        .layer(middleware::from_fn(rate_limit::rate_limit_read))
        .layer(axum::Extension(cache_arc.clone()))
        .with_state(state.clone());

    let health_routes = Router::new()
        .route("/api/health", get(handle_health))
        .with_state(state.clone());

    let app = Router::new()
        .merge(health_routes)
        .merge(withdraw_routes)
        .merge(read_routes)
        .layer(CorsLayer::permissive());

    let port = std::env::var("PORT").unwrap_or_else(|_| "4001".to_string());
    let addr = format!("0.0.0.0:{}", port);

    println!("🚀 Backend server running on http://{}", addr);
    println!("📡 Endpoints:");
    println!("  POST /api/withdraw    - Proxy to prover service (5 req/min)");
    println!("  GET  /api/commitments - List all commitments (60 req/min)");
    println!("  GET  /api/root        - Get latest root (60 req/min)");
    println!("  GET  /api/proof       - Get Merkle proof (60 req/min)");
    println!("  GET  /api/health      - Health check (unlimited)");

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
