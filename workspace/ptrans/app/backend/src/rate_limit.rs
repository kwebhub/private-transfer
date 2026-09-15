use crate::cache::Cache;
use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::{Response, StatusCode},
    middleware::Next,
};
use std::net::SocketAddr;
use std::sync::Arc;

/// Лимиты для разных эндпоинтов (requests per minute)
const WITHDRAW_LIMIT_PER_MIN: u64 = 5;
const READ_LIMIT_PER_MIN: u64 = 60;

/// Middleware: rate limiting для /api/withdraw
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

    let key = format!("ratelimit:withdraw:{}", addr.ip());

    let count = cache
        .incr_rate_limit(&key, 60)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if count > WITHDRAW_LIMIT_PER_MIN {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

/// Middleware: rate limiting для чтения (commitments, root, proof)
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

    let key = format!("ratelimit:read:{}", addr.ip());

    let count = cache
        .incr_rate_limit(&key, 60)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if count > READ_LIMIT_PER_MIN {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}
