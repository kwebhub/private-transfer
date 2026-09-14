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

    /// Инвалидировать кеш для указанного pool (commitments, root)
    pub async fn invalidate_pool(&self, pool_address: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.conn.clone();
        let commitments_key = format!("commitments:{}", pool_address);
        let root_key = format!("root:{}", pool_address);
        conn.del::<_, ()>(vec![commitments_key, root_key]).await
    }
}
