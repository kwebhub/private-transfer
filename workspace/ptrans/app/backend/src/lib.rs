//! zk-pool backend library.
//!
//! Этот crate содержит **всю** логику. `main.rs` — только bootstrap.
//! Это позволяет писать unit- и integration-тесты.

pub mod cache;
pub mod config;
pub mod db;
pub mod indexer;
pub mod metrics;
pub mod rate_limit;
pub mod tree;
