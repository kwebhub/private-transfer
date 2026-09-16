//! Rate limiting middleware для axum.
//!
//! Лимиты читаются из `Config` (env).
//! Счётчики хранятся в Redis (INCR + EXPIRE).

use crate::cache::Cache;
use crate::config::Config;
use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::{Response, StatusCode},
    middleware::Next,
};
use std::net::SocketAddr;
use std::sync::Arc;

/// Лимит для `/api/withdraw` (запросов в минуту на IP).
fn withdraw_limit(config: &Config) -> u64 {
    config.rate_limit_withdraw_per_min
}

/// Лимит для чтения (запросов в минуту на IP).
fn read_limit(config: &Config) -> u64 {
    config.rate_limit_read_per_min
}

/// Middleware: rate limiting для /api/withdraw.
pub async fn rate_limit_withdraw(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let cache = request
        .extensions()
        .get::<Arc<Cache>>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let config = request
        .extensions()
        .get::<Arc<Config>>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let key = format!("ratelimit:withdraw:{}", addr.ip());

    let count = cache
        .incr_rate_limit(&key, 60)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if count > withdraw_limit(&config) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

/// Middleware: rate limiting для чтения (commitments, root, proof).
pub async fn rate_limit_read(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let cache = request
        .extensions()
        .get::<Arc<Cache>>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let config = request
        .extensions()
        .get::<Arc<Config>>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let key = format!("ratelimit:read:{}", addr.ip());

    let count = cache
        .incr_rate_limit(&key, 60)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if count > read_limit(&config) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}
