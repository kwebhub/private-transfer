use crate::cache::Cache;
use crate::db::Db;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

const POLL_INTERVAL_SECS: u64 = 5;

pub struct Indexer {
    db: Arc<Db>,
    cache: Arc<Cache>,
    rpc_url: String,
    pool_address: String,
}

impl Indexer {
    pub fn new(db: Arc<Db>, cache: Arc<Cache>, rpc_url: String, pool_address: String) -> Self {
        Self {
            db,
            cache,
            rpc_url,
            pool_address,
        }
    }

    pub async fn run(self) {
        println!("🔄 Indexer started for pool {}", self.pool_address);

        loop {
            if let Err(e) = self.tick().await {
                eprintln!("⚠️ Indexer tick error: {}", e);
            }
            sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }
    }

    async fn tick(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let sigs_resp: serde_json::Value = client
            .post(&self.rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getSignaturesForAddress",
                "params": [self.pool_address, { "limit": 50 }]
            }))
            .send()
            .await?
            .json()
            .await?;

        let sigs = sigs_resp["result"].as_array().cloned().unwrap_or_default();

        for sig_entry in sigs {
            let signature = match sig_entry["signature"].as_str() {
                Some(s) => s,
                None => continue,
            };

            if let Err(e) = self.process_transaction(&client, signature).await {
                eprintln!("⚠️ Failed to process {}: {}", signature, e);
            }
        }

        Ok(())
    }

    async fn process_transaction(
        &self,
        client: &reqwest::Client,
        signature: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tx_resp: serde_json::Value = client
            .post(&self.rpc_url)
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

            // DepositEvent: discriminator(8) + commitment(32) + leaf_index(8) + timestamp(8) + new_root(32) = 88 байт
            if bytes.len() == 88 {
                let commitment = &bytes[8..40];
                let leaf_index = i64::from_le_bytes(bytes[40..48].try_into().unwrap());
                let new_root = &bytes[56..88];

                if !self.db.commitment_exists(leaf_index).await? {
                    self.db
                        .save_deposit(
                            leaf_index,
                            commitment,
                            new_root,
                            &self.pool_address,
                            signature,
                        )
                        .await?;

                    // Инвалидируем кеш
                    let _ = self.cache.invalidate_pool(&self.pool_address).await;

                    println!(
                        "📥 Indexed deposit: leaf={} commitment={}",
                        leaf_index,
                        hex::encode(commitment)
                    );
                }
            }

            // WithdrawEvent: discriminator(8) + nullifier_hash(32) + recipient(32) + timestamp(8) = 80 байт
            if bytes.len() == 80 {
                let nullifier_hash = &bytes[8..40];
                let recipient = &bytes[40..72];

                if !self.db.is_nullifier_used(nullifier_hash).await? {
                    self.db
                        .save_nullifier(
                            nullifier_hash,
                            &self.pool_address,
                            &bs58::encode(recipient).into_string(),
                            0,
                            signature,
                        )
                        .await?;

                    println!(
                        "📤 Indexed withdraw: nullifier={}",
                        hex::encode(nullifier_hash)
                    );
                }
            }
        }

        Ok(())
    }
}
