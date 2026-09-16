//! Indexer: слушает события `DepositEvent` и `WithdrawEvent`,
//! пишет их в Postgres, обновляет Merkle Tree в Redis.
//!
//! ## Как работает
//!
//! 1. Каждые `INDEXER_POLL_INTERVAL_SECS` секунд вызывает `getSignaturesForAddress`.
//! 2. Идёт по пагинации (`before`) до `last_signature` (в Redis).
//! 3. Разворачивает — от старых к новым.
//! 4. Для каждой tx парсит логи: `DepositEvent` (88 байт) / `WithdrawEvent` (80 байт).
//! 5. Дедупликация: `commitment_exists` / `is_nullifier_used`.
//! 6. При новом депозите — `tree::add_leaf` обновляет дерево.
//! 7. Запоминает `last_signature`.
//!
//! Все параметры (poll interval, page size, max pages) — из `Config`.

use crate::cache::Cache;
use crate::config::Config;
use crate::db::Db;
use crate::metrics;
use crate::tree;
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};
use tracing::{debug, error, info, warn};

/// Indexer пула: фоновый воркер для синхронизации с блокчейном.
pub struct Indexer {
    db: Arc<Db>,
    cache: Arc<Cache>,
    config: Arc<Config>,
}

impl Indexer {
    /// Создаёт новый indexer.
    pub fn new(db: Arc<Db>, cache: Arc<Cache>, config: Arc<Config>) -> Self {
        Self { db, cache, config }
    }

    /// Запускает бесконечный цикл обработки событий.
    ///
    /// Никогда не возвращается, кроме паники.
    pub async fn run(self) {
        info!(pool = %self.config.pool_address, "🔄 Indexer started");

        loop {
            let start = Instant::now();
            if let Err(e) = self.tick().await {
                error!(error = %e, "indexer tick error");
                metrics::record_indexer_error();
            }
            metrics::observe_indexer_tick(start.elapsed().as_secs_f64());
            sleep(Duration::from_secs(self.config.indexer_poll_interval_secs)).await;
        }
    }

    async fn tick(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let last_processed = self
            .cache
            .get_last_signature(&self.config.pool_address)
            .await
            .unwrap_or(None);

        let mut before: Option<String> = None;
        let mut all_sigs: Vec<serde_json::Value> = Vec::new();
        let mut reached_last = false;

        for _page in 0..self.config.indexer_max_pages {
            let mut params = serde_json::json!([
                self.config.pool_address,
                { "limit": self.config.indexer_page_size }
            ]);
            if let Some(b) = &before {
                params[1]["before"] = serde_json::json!(b);
            }

            let sigs_resp: serde_json::Value = client
                .post(&self.config.solana_rpc_url)
                .json(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "getSignaturesForAddress",
                    "params": params
                }))
                .send()
                .await?
                .json()
                .await?;

            let page_sigs = sigs_resp["result"].as_array().cloned().unwrap_or_default();

            if page_sigs.is_empty() {
                break;
            }

            let mut stop_pagination = false;

            for sig_entry in &page_sigs {
                let signature = match sig_entry["signature"].as_str() {
                    Some(s) => s,
                    None => continue,
                };

                if let Some(last) = &last_processed {
                    if signature == last {
                        reached_last = true;
                        stop_pagination = true;
                        break;
                    }
                }

                all_sigs.push(sig_entry.clone());
            }

            if stop_pagination {
                break;
            }

            if let Some(last_sig) = page_sigs.last() {
                before = last_sig["signature"].as_str().map(|s| s.to_string());
            } else {
                break;
            }
        }

        if let Some(first) = all_sigs.first() {
            if let Some(block_time) = first["blockTime"].as_i64() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let lag = (now - block_time).max(0) as f64;
                metrics::record_indexer_lag(lag);
            }
        }

        if all_sigs.is_empty() {
            return Ok(());
        }

        all_sigs.reverse();

        let mut newest_signature: Option<String> = None;

        for sig_entry in &all_sigs {
            let signature = match sig_entry["signature"].as_str() {
                Some(s) => s,
                None => continue,
            };

            if let Err(e) = self.process_transaction(&client, signature).await {
                warn!(signature = %signature, error = %e, "failed to process transaction");
                metrics::record_indexer_error();
                continue;
            }

            newest_signature = Some(signature.to_string());
        }

        if let Some(newest) = newest_signature {
            let _ = self
                .cache
                .set_last_signature(&self.config.pool_address, &newest)
                .await;

            if reached_last {
                debug!(signature = %&newest[..8], "indexer caught up");
            } else {
                info!(
                    count = all_sigs.len(),
                    signature = %&newest[..8],
                    "indexer processed new transactions"
                );
            }
        }

        Ok(())
    }

    async fn process_transaction(
        &self,
        client: &reqwest::Client,
        signature: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let tx_resp: serde_json::Value = client
            .post(&self.config.solana_rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getTransaction",
                "params": [signature, {
                    "encoding": "json",
                    "maxSupportedTransactionVersion": 0
                }]
            }))
            .send()
            .await?
            .json()
            .await?;

        let logs = match tx_resp["result"]["meta"]["logMessages"].as_array() {
            Some(l) => l,
            None => return Ok(()),
        };

        for log in logs {
            let log_str = match log.as_str() {
                Some(s) => s,
                None => continue,
            };

            if !log_str.starts_with("Program data:") {
                continue;
            }

            let b64 = log_str.replace("Program data: ", "").trim().to_string();
            let bytes =
                match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &b64) {
                    Ok(b) => b,
                    Err(_) => continue,
                };

            // DepositEvent
            if bytes.len() == 88 {
                let commitment = &bytes[8..40];
                let leaf_index = i64::from_le_bytes(bytes[40..48].try_into().unwrap());
                let new_root = &bytes[56..88];
                let commitment_hex = hex::encode(commitment);

                if !self.db.commitment_exists(leaf_index).await? {
                    self.db
                        .save_deposit(
                            leaf_index,
                            commitment,
                            new_root,
                            &self.config.pool_address,
                            signature,
                        )
                        .await?;

                    match tree::add_leaf(
                        &self.cache,
                        &self.config.merkle_url,
                        &self.config.pool_address,
                        leaf_index as usize,
                        &commitment_hex,
                        self.config.merkle_tree_depth,
                    )
                    .await
                    {
                        Ok(root) => {
                            info!(
                                leaf = leaf_index,
                                commitment = %commitment_hex,
                                root = %root,
                                "📥 Indexed deposit"
                            );
                            metrics::record_deposit(leaf_index);
                        }
                        Err(e) => {
                            error!(error = %e, "failed to add leaf to tree");
                            metrics::record_tree_error();
                        }
                    }

                    let _ = self.cache.invalidate_pool(&self.config.pool_address).await;
                }
            }

            // WithdrawEvent
            if bytes.len() == 80 {
                let nullifier_hash = &bytes[8..40];
                let recipient = &bytes[40..72];

                if !self.db.is_nullifier_used(nullifier_hash).await? {
                    self.db
                        .save_nullifier(
                            nullifier_hash,
                            &self.config.pool_address,
                            &bs58::encode(recipient).into_string(),
                            0,
                            signature,
                        )
                        .await?;

                    info!(
                        nullifier = %hex::encode(nullifier_hash),
                        "📤 Indexed withdraw"
                    );
                    metrics::record_withdrawal();
                }
            }
        }

        Ok(())
    }
}
