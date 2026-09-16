//! zk-pool backend library.
//!
//! Этот crate содержит **всю** логику. `main.rs` — только bootstrap.
//! Это позволяет писать unit- и integration-тесты.
//!
//! ## Модули
//!
//! - [`config`] — конфигурация из env.
//! - [`db`] — Postgres (commitments, roots, nullifiers).
//! - [`cache`] — Redis (кеш, Merkle tree, rate limit).
//! - [`tree`] — инкрементальное Merkle Tree в Redis.
//! - [`indexer`] — фоновый воркер (события → БД → дерево).
//! - [`logging`] — инициализация `tracing`.
//! - [`metrics`] — Prometheus-метрики.
//! - [`rate_limit`] — middleware rate limiting.

pub mod cache;
pub mod config;
pub mod db;
pub mod indexer;
pub mod logging;
pub mod metrics;
pub mod rate_limit;
pub mod tree;
