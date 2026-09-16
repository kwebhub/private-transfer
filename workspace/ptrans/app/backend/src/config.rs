//! Конфигурация backend.
//!
//! Все значения читаются из переменных окружения.
//! Если переменная не задана и не имеет дефолта — приложение падает при старте.

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    // ============ Обязательные ============
    pub database_url: String,
    pub redis_url: String,

    // ============ С дефолтами ============
    pub port: u16,
    pub prover_url: String,
    pub merkle_url: String,
    pub solana_rpc_url: String,
    pub pool_address: String,

    // ============ Тюнинг ============
    pub indexer_poll_interval_secs: u64,
    pub indexer_page_size: usize,
    pub indexer_max_pages: usize,
    pub rate_limit_withdraw_per_min: u64,
    pub rate_limit_read_per_min: u64,
    pub merkle_tree_depth: u32,
}

impl Config {
    /// Загружает конфигурацию из переменных окружения.
    ///
    /// - `DATABASE_URL` — обязательна
    /// - `REDIS_URL` — дефолт `redis://localhost:6379`
    /// - `PORT` — дефолт `4001`
    /// - `PROVER_URL` — дефолт `http://localhost:4002`
    /// - `MERKLE_URL` — дефолт `http://localhost:4003`
    /// - `SOLANA_RPC_URL` — дефолт `https://api.devnet.solana.com`
    /// - `POOL_ADDRESS` — обязательна
    /// - `INDEXER_POLL_INTERVAL_SECS` — дефолт `5`
    /// - `INDEXER_PAGE_SIZE` — дефолт `100`
    /// - `INDEXER_MAX_PAGES` — дефолт `10`
    /// - `RATE_LIMIT_WITHDRAW_PER_MIN` — дефолт `5`
    /// - `RATE_LIMIT_READ_PER_MIN` — дефолт `60`
    /// - `MERKLE_TREE_DEPTH` — дефолт `20`
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            redis_url: optional("REDIS_URL", "redis://localhost:6379"),
            port: parse("PORT", 4001u16)?,
            prover_url: optional("PROVER_URL", "http://localhost:4002"),
            merkle_url: optional("MERKLE_URL", "http://localhost:4003"),
            solana_rpc_url: optional("SOLANA_RPC_URL", "https://api.devnet.solana.com"),
            pool_address: required("POOL_ADDRESS")?,
            indexer_poll_interval_secs: parse("INDEXER_POLL_INTERVAL_SECS", 5u64)?,
            indexer_page_size: parse("INDEXER_PAGE_SIZE", 100usize)?,
            indexer_max_pages: parse("INDEXER_MAX_PAGES", 10usize)?,
            rate_limit_withdraw_per_min: parse("RATE_LIMIT_WITHDRAW_PER_MIN", 5u64)?,
            rate_limit_read_per_min: parse("RATE_LIMIT_READ_PER_MIN", 60u64)?,
            merkle_tree_depth: parse("MERKLE_TREE_DEPTH", 20u32)?,
        })
    }
}

/// Ошибка конфигурации.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    Missing(String),

    #[error("Invalid value for {name}: {value} (expected {expected})")]
    Invalid {
        name: String,
        value: String,
        expected: String,
    },
}

/// Читает обязательную переменную. Если не задана — ошибка.
fn required(name: &str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::Missing(name.to_string()))
}

/// Читает переменную или использует дефолт.
fn optional(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

/// Читает переменную и парсит её. Использует дефолт, если не задана.
fn parse<T>(name: &str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr + std::fmt::Display + Copy,
    T::Err: std::fmt::Display,
{
    match env::var(name) {
        Ok(v) => v.parse::<T>().map_err(|e| ConfigError::Invalid {
            name: name.to_string(),
            value: v.clone(),
            expected: format!("{} ({})", std::any::type_name::<T>(), e),
        }),
        Err(_) => Ok(default),
    }
}
