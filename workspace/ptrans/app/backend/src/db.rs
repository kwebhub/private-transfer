//! Обёртка над Postgres.
//!
//! Источник истины для:
//! - `commitments` — все листья Merkle Tree.
//! - `roots` — история корней (для проверки старых proof).
//! - `nullifiers` — использованные нуллификаторы (защита от double-spend).

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Обёртка над пулом соединений Postgres.
#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    /// Создаёт пул соединений и проверяет подключение.
    ///
    /// Максимум — 10 соединений в пуле.
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    /// Возвращает ссылку на пул для прямых запросов.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Сохраняет депозит в БД (commitments + roots).
    ///
    /// Транзакционно: либо оба INSERT'а, либо ничего.
    /// Идемпотентно — повторный INSERT игнорируется (`ON CONFLICT DO NOTHING`).
    ///
    /// # Параметры
    ///
    /// - `leaf_index` — индекс листа в Merkle Tree.
    /// - `commitment` — 32 байта (Poseidon2-хеш).
    /// - `new_root` — корень после добавления листа.
    /// - `pool_address` — адрес PoolAcc PDA (Base58).
    /// - `tx_signature` — сигнатура транзакции депозита.
    pub async fn save_deposit(
        &self,
        leaf_index: i64,
        commitment: &[u8],
        new_root: &[u8],
        pool_address: &str,
        tx_signature: &str,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO commitments (leaf_index, commitment, pool_address, tx_signature)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (leaf_index) DO NOTHING
            "#,
        )
        .bind(leaf_index)
        .bind(commitment)
        .bind(pool_address)
        .bind(tx_signature)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO roots (root, leaf_index, pool_address, tx_signature)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(new_root)
        .bind(leaf_index)
        .bind(pool_address)
        .bind(tx_signature)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Проверяет, использован ли нуллификатор.
    pub async fn is_nullifier_used(&self, nullifier_hash: &[u8]) -> Result<bool, sqlx::Error> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM nullifiers WHERE nullifier_hash = $1")
                .bind(nullifier_hash)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }

    /// Проверяет, записан ли commitment с таким `leaf_index`.
    pub async fn commitment_exists(&self, leaf_index: i64) -> Result<bool, sqlx::Error> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM commitments WHERE leaf_index = $1")
                .bind(leaf_index)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }

    /// Записывает использованный нуллификатор.
    ///
    /// Идемпотентно — повторный INSERT игнорируется.
    pub async fn save_nullifier(
        &self,
        nullifier_hash: &[u8],
        pool_address: &str,
        recipient: &str,
        amount: i64,
        tx_signature: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO nullifiers (nullifier_hash, pool_address, recipient, amount, tx_signature)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (nullifier_hash) DO NOTHING
            "#,
        )
        .bind(nullifier_hash)
        .bind(pool_address)
        .bind(recipient)
        .bind(amount)
        .bind(tx_signature)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Возвращает все commitments для пула, отсортированные по `leaf_index`.
    pub async fn get_commitments(
        &self,
        pool_address: &str,
    ) -> Result<Vec<(i64, Vec<u8>)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (i64, Vec<u8>)>(
            "SELECT leaf_index, commitment FROM commitments WHERE pool_address = $1 ORDER BY leaf_index ASC",
        )
        .bind(pool_address)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Возвращает последний добавленный root.
    pub async fn get_latest_root(
        &self,
        pool_address: &str,
    ) -> Result<Option<Vec<u8>>, sqlx::Error> {
        let row = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT root FROM roots WHERE pool_address = $1 ORDER BY id DESC LIMIT 1",
        )
        .bind(pool_address)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }
}
