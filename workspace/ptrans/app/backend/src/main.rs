//! zk-pool backend binary.
//!
//! Точка входа: загружает конфиг, инициализирует tracing/metrics/DB/Redis,
//! синхронизирует Merkle Tree, запускает indexer и HTTP-сервер.
//!
//! ## Endpoints
//!
//! - `POST /api/withdraw` — проксирует witness в prover (5 req/min/IP).
//! - `GET  /api/commitments` — список commitments (60 req/min/IP).
//! - `GET  /api/root` — текущий root (60 req/min/IP).
//! - `GET  /api/proof` — Merkle proof (60 req/min/IP).
//! - `GET  /api/health` — health check.
//! - `GET  /metrics` — Prometheus метрики.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
    Json, Router,
};
use axum_prometheus::PrometheusMetricLayer;
use metrics_exporter_prometheus::PrometheusBuilder;
use ptrans_backend::{cache, config::Config, db, indexer, logging, metrics, rate_limit, tree};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};

// ============ СТРУКТУРЫ ============

#[derive(Clone)]
struct AppState {
    db: Arc<db::Db>,
    cache: Arc<cache::Cache>,
    config: Arc<Config>,
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
        Err(_) => {
            metrics::record_db_error();
            "error"
        }
    };

    Json(serde_json::json!({
        "status": "ok",
        "db": db_status,
    }))
}

async fn handle_withdraw(
    State(state): State<AppState>,
    Json(payload): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, (StatusCode, String)> {
    let url = format!("{}/prove", state.config.prover_url);

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "witness": payload.witness }))
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "prover request failed");
            (
                StatusCode::BAD_GATEWAY,
                format!("Prover request failed: {}", e),
            )
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        warn!(status = %status, "prover returned error");
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("Prover error {}: {}", status, body),
        ));
    }

    let prover_response: WithdrawResponse = resp.json().await.map_err(|e| {
        error!(error = %e, "invalid prover response");
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
            error!(error = %e, "db error");
            metrics::record_db_error();
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
            error!(error = %e, "db error");
            metrics::record_db_error();
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
    match tree::get_proof(
        &state.cache,
        &query.pool_address,
        query.leaf_index,
        state.config.merkle_tree_depth,
    )
    .await
    {
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
        Err(e) => {
            error!(error = %e, "tree error");
            metrics::record_tree_error();
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Tree error: {}", e),
            ))
        }
    }
}

// ============ MAIN ============

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // `.env` лежит в workspace/ptrans/.env — на 2 уровня выше app/backend/
    dotenvy::from_filename("../../.env").ok();
    dotenvy::dotenv().ok();

    // Инициализация tracing (использует RUST_LOG)
    logging::init();
    info!("⚙️  Logging initialized");

    let config = Arc::new(Config::from_env()?);
    info!(port = config.port, "Config loaded");

    let prometheus_handle = PrometheusBuilder::new()
        .install_recorder()
        .map_err(|e| format!("Failed to install Prometheus recorder: {}", e))?;

    metrics::init_metrics();
    info!("📊 Metrics initialized");

    let db = db::Db::new(&config.database_url).await?;
    info!("✅ Connected to Postgres");

    let cache = cache::Cache::new(&config.redis_url).await?;
    info!("✅ Connected to Redis");

    let db_arc = Arc::new(db);
    let cache_arc = Arc::new(cache);

    // Синхронизация дерева
    let tree_size = cache_arc
        .get_tree_size(&config.pool_address)
        .await?
        .unwrap_or(0);
    let db_commitments = db_arc.get_commitments(&config.pool_address).await?;

    if tree_size == 0
        && cache_arc
            .get_tree_root(&config.pool_address)
            .await?
            .is_none()
    {
        info!("🌳 Initializing empty tree in Redis...");
        tree::init_empty_tree(
            &cache_arc,
            &config.merkle_url,
            &config.pool_address,
            config.merkle_tree_depth,
        )
        .await
        .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
    }

    let current_size = cache_arc
        .get_tree_size(&config.pool_address)
        .await?
        .unwrap_or(0);

    if db_commitments.len() > current_size {
        info!(
            db_len = db_commitments.len(),
            redis_len = current_size,
            "🌳 Syncing tree"
        );

        for (leaf_index, commitment) in &db_commitments[current_size..] {
            let commitment_hex = hex::encode(commitment);
            tree::add_leaf(
                &cache_arc,
                &config.merkle_url,
                &config.pool_address,
                *leaf_index as usize,
                &commitment_hex,
                config.merkle_tree_depth,
            )
            .await
            .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;
        }

        let final_root = cache_arc
            .get_tree_root(&config.pool_address)
            .await?
            .unwrap_or_else(|| "unknown".to_string());
        info!(root = %final_root, "🌳 Tree synced");
    } else {
        let root = cache_arc
            .get_tree_root(&config.pool_address)
            .await?
            .unwrap_or_else(|| "unknown".to_string());
        info!(
            leaves = current_size,
            root = %root,
            "🌳 Tree already in sync"
        );
    }

    if current_size > 0 {
        metrics::record_deposit(current_size as i64 - 1);
        info!(size = current_size, "📊 Recorded tree size");
    }

    let state = AppState {
        db: db_arc.clone(),
        cache: cache_arc.clone(),
        config: config.clone(),
    };

    let indexer = indexer::Indexer::new(db_arc.clone(), cache_arc.clone(), config.clone());
    tokio::spawn(async move {
        indexer.run().await;
    });

    let (prometheus_layer, _metric_handle) = PrometheusMetricLayer::pair();

    let withdraw_routes = Router::new()
        .route("/api/withdraw", post(handle_withdraw))
        .layer(middleware::from_fn(rate_limit::rate_limit_withdraw))
        .layer(axum::Extension(cache_arc.clone()))
        .layer(axum::Extension(config.clone()))
        .with_state(state.clone());

    let read_routes = Router::new()
        .route("/api/commitments", get(handle_commitments))
        .route("/api/root", get(handle_root))
        .route("/api/proof", get(handle_proof))
        .layer(middleware::from_fn(rate_limit::rate_limit_read))
        .layer(axum::Extension(cache_arc.clone()))
        .layer(axum::Extension(config.clone()))
        .with_state(state.clone());

    let health_routes = Router::new()
        .route("/api/health", get(handle_health))
        .with_state(state.clone());

    let metrics_handle_clone = prometheus_handle.clone();
    let metrics_route = Router::new().route(
        "/metrics",
        get(move || {
            let handle = metrics_handle_clone.clone();
            async move { handle.render() }
        }),
    );

    let app = Router::new()
        .merge(health_routes)
        .merge(withdraw_routes)
        .merge(read_routes)
        .merge(metrics_route)
        .layer(prometheus_layer)
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", config.port);

    info!(%addr, "🚀 Backend server running");
    info!("📡 Endpoints:");
    info!("  POST /api/withdraw    - Proxy to prover service");
    info!("  GET  /api/commitments - List all commitments (cached)");
    info!("  GET  /api/root        - Get latest root (from Redis tree)");
    info!("  GET  /api/proof       - Get Merkle proof (from Redis tree)");
    info!("  GET  /api/health      - Health check");
    info!("  GET  /metrics         - Prometheus metrics");

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
