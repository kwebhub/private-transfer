use redis::aio::ConnectionManager;
use redis::AsyncCommands;

#[derive(Clone)]
pub struct Cache {
    conn: ConnectionManager,
}

impl Cache {
    pub async fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }

    /// Установить строковое значение с TTL (секунды)
    pub async fn set_ex(
        &self,
        key: &str,
        value: &str,
        ttl_secs: u64,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        conn.set_ex::<_, _, ()>(key, value, ttl_secs).await
    }

    /// Установить строковое значение без TTL
    pub async fn set(&self, key: &str, value: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        conn.set::<_, _, ()>(key, value).await
    }

    /// Получить строковое значение
    pub async fn get(&self, key: &str) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.conn.clone();
        conn.get(key).await
    }

    /// Удалить ключ
    pub async fn del(&self, key: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        conn.del::<_, ()>(key).await
    }

    /// Инвалидировать кеш для указанного pool
    pub async fn invalidate_pool(&self, pool_address: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        let keys = vec![
            format!("commitments:{}", pool_address),
            format!("root:{}", pool_address),
        ];
        conn.del::<_, ()>(keys).await
    }

    // ============================================================
    // Merkle Tree методы
    // ============================================================

    /// Получить уровень дерева (JSON-массив hex-хешей)
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

    /// Записать уровень дерева
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

    /// Получить root из кеша дерева
    pub async fn get_tree_root(
        &self,
        pool_address: &str,
    ) -> Result<Option<String>, redis::RedisError> {
        let key = format!("tree:{}:root", pool_address);
        self.get(&key).await
    }

    /// Записать root в кеш дерева
    pub async fn set_tree_root(
        &self,
        pool_address: &str,
        root: &str,
    ) -> Result<(), redis::RedisError> {
        let key = format!("tree:{}:root", pool_address);
        self.set(&key, root).await
    }

    /// Получить количество листьев в дереве
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

    /// Записать количество листьев
    pub async fn set_tree_size(
        &self,
        pool_address: &str,
        size: usize,
    ) -> Result<(), redis::RedisError> {
        let key = format!("tree:{}:size", pool_address);
        self.set(&key, &size.to_string()).await
    }

    /// Полностью очистить дерево в кеше
    pub async fn clear_tree(&self, pool_address: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();

        let mut keys = vec![
            format!("tree:{}:root", pool_address),
            format!("tree:{}:size", pool_address),
        ];
        // 21 уровень (0..20) + корень
        for d in 0..=20 {
            keys.push(format!("tree:{}:level:{}", pool_address, d));
        }

        conn.del::<_, ()>(keys).await
    }

    /// Получить empty hash для уровня
    pub async fn get_empty_hash(
        &self,
        pool_address: &str,
        level: u32,
    ) -> Result<Option<String>, redis::RedisError> {
        let key = format!("tree:{}:empty:{}", pool_address, level);
        self.get(&key).await
    }

    /// Записать empty hash для уровня
    pub async fn set_empty_hash(
        &self,
        pool_address: &str,
        level: u32,
        hash: &str,
    ) -> Result<(), redis::RedisError> {
        let key = format!("tree:{}:empty:{}", pool_address, level);
        self.set(&key, hash).await
    }

    /// Получить последнюю обработанную сигнатуру для pool
    pub async fn get_last_signature(
        &self,
        pool_address: &str,
    ) -> Result<Option<String>, redis::RedisError> {
        let key = format!("indexer:{}:last_signature", pool_address);
        self.get(&key).await
    }

    /// Записать последнюю обработанную сигнатуру для pool
    pub async fn set_last_signature(
        &self,
        pool_address: &str,
        signature: &str,
    ) -> Result<(), redis::RedisError> {
        let key = format!("indexer:{}:last_signature", pool_address);
        self.set(&key, signature).await
    }

    /// Инкрементировать счётчик запросов и получить текущее значение.
    /// TTL устанавливается при первом запросе.
    pub async fn incr_rate_limit(
        &self,
        key: &str,
        ttl_secs: u64,
    ) -> Result<u64, redis::RedisError> {
        let mut conn = self.conn.clone();

        // INCR возвращает новое значение
        let count: u64 = conn.incr(key, 1u64).await?;

        // Устанавливаем TTL только при первом запросе (count == 1)
        if count == 1 {
            conn.expire::<_, ()>(key, ttl_secs as i64).await?;
        }

        Ok(count)
    }
}
