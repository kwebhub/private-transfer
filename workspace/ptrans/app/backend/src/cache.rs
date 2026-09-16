//! Обёртка над Redis.
//!
//! Используется для:
//! - **Кеша commitments** — `/api/commitments` (TTL 30 сек).
//! - **Merkle Tree** — уровни, пустые хеши, корень, размер.
//! - **Indexer** — последняя обработанная сигнатура.
//! - **Rate limiting** — счётчики запросов по IP.

use redis::aio::ConnectionManager;
use redis::AsyncCommands;

/// Обёртка над `ConnectionManager` Redis.
#[derive(Clone)]
pub struct Cache {
    conn: ConnectionManager,
}

impl Cache {
    /// Создаёт соединение с Redis.
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }

    /// Устанавливает значение с TTL (в секундах).
    pub async fn set_ex(
        &self,
        key: &str,
        value: &str,
        ttl_secs: u64,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        conn.set_ex::<_, _, ()>(key, value, ttl_secs).await
    }

    /// Устанавливает значение без TTL.
    pub async fn set(&self, key: &str, value: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        conn.set::<_, _, ()>(key, value).await
    }

    /// Читает значение. Возвращает `None`, если ключа нет.
    pub async fn get(&self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.conn.clone();
        conn.get(key).await
    }

    /// Инвалидирует кеш пула: `commitments:{pool}` и `root:{pool}`.
    pub async fn invalidate_pool(&self, pool_address: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        let keys = vec![
            format!("commitments:{}", pool_address),
            format!("root:{}", pool_address),
        ];
        conn.del::<_, ()>(keys).await
    }

    // ============================================================
    // Merkle Tree
    // ============================================================

    /// Возвращает уровень дерева (`tree:{pool}:level:{d}`) как массив hex-хешей.
    pub async fn get_tree_level(
        &self,
        pool_address: &str,
        level: u32,
    ) -> Result<Option<Vec<String>>, redis::RedisError> {
        let key = format!("tree:{}:level:{}", pool_address, level);
        match self.get(&key).await? {
            Some(json) => {
                let parsed: Vec<String> = serde_json::from_str(&json).unwrap_or_default();
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// Записывает уровень дерева.
    pub async fn set_tree_level(
        &self,
        pool_address: &str,
        level: u32,
        hashes: &[String],
    ) -> Result<(), redis::RedisError> {
        let key = format!("tree:{}:level:{}", pool_address, level);
        let json = serde_json::to_string(hashes).unwrap_or_else(|_| "[]".to_string());
        self.set(&key, &json).await
    }

    /// Возвращает текущий корень (`tree:{pool}:root`).
    pub async fn get_tree_root(
        &self,
        pool_address: &str,
    ) -> Result<Option<String>, redis::RedisError> {
        let key = format!("tree:{}:root", pool_address);
        self.get(&key).await
    }

    /// Записывает текущий корень.
    pub async fn set_tree_root(
        &self,
        pool_address: &str,
        root: &str,
    ) -> Result<(), redis::RedisError> {
        let key = format!("tree:{}:root", pool_address);
        self.set(&key, root).await
    }

    /// Возвращает количество листьев (`tree:{pool}:size`).
    pub async fn get_tree_size(
        &self,
        pool_address: &str,
    ) -> Result<Option<usize>, redis::RedisError> {
        let key = format!("tree:{}:size", pool_address);
        match self.get(&key).await? {
            Some(s) => Ok(s.parse().ok()),
            None => Ok(None),
        }
    }

    /// Записывает количество листьев.
    pub async fn set_tree_size(
        &self,
        pool_address: &str,
        size: usize,
    ) -> Result<(), redis::RedisError> {
        let key = format!("tree:{}:size", pool_address);
        self.set(&key, &size.to_string()).await
    }

    /// Возвращает "пустой" хеш для уровня (`tree:{pool}:empty:{d}`).
    pub async fn get_empty_hash(
        &self,
        pool_address: &str,
        level: u32,
    ) -> Result<Option<String>, redis::RedisError> {
        let key = format!("tree:{}:empty:{}", pool_address, level);
        self.get(&key).await
    }

    /// Записывает "пустой" хеш для уровня.
    pub async fn set_empty_hash(
        &self,
        pool_address: &str,
        level: u32,
        hash: &str,
    ) -> Result<(), redis::RedisError> {
        let key = format!("tree:{}:empty:{}", pool_address, level);
        self.set(&key, hash).await
    }

    // ============================================================
    // Indexer
    // ============================================================

    /// Возвращает последнюю обработанную сигнатуру (`indexer:{pool}:last_signature`).
    pub async fn get_last_signature(
        &self,
        pool_address: &str,
    ) -> Result<Option<String>, redis::RedisError> {
        let key = format!("indexer:{}:last_signature", pool_address);
        self.get(&key).await
    }

    /// Записывает последнюю обработанную сигнатуру.
    pub async fn set_last_signature(
        &self,
        pool_address: &str,
        signature: &str,
    ) -> Result<(), redis::RedisError> {
        let key = format!("indexer:{}:last_signature", pool_address);
        self.set(&key, signature).await
    }

    // ============================================================
    // Rate limiting
    // ============================================================

    /// Инкрементирует счётчик и возвращает новое значение.
    ///
    /// TTL устанавливается **только** при первом запросе (`INCR` вернул 1).
    pub async fn incr_rate_limit(
        &self,
        key: &str,
        ttl_secs: u64,
    ) -> Result<u64, redis::RedisError> {
        let mut conn = self.conn.clone();
        let count: u64 = conn.incr(key, 1u64).await?;
        if count == 1 {
            conn.expire::<_, ()>(key, ttl_secs as i64).await?;
        }
        Ok(count)
    }
}
