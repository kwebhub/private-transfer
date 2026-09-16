//! Integration tests для `ptrans_backend::db::Db`.
//!
//! Требуют работающего Postgres на `DATABASE_URL`.
//! При отсутствии БД — тесты пропускаются (возвращают `Ok`).

use ptrans_backend::db::Db;

/// Возвращает URL для тестовой БД.
/// Если `TEST_DATABASE_URL` не установлен — используется дефолтный.
fn test_db_url() -> String {
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://ptrans:ptrans_dev_password@localhost:5432/ptrans".to_string()
    })
}

/// Подключается к БД или пропускает тест.
async fn setup() -> Option<Db> {
    match Db::new(&test_db_url()).await {
        Ok(db) => Some(db),
        Err(e) => {
            eprintln!("⚠️ Skipping test: cannot connect to Postgres: {}", e);
            None
        }
    }
}

/// Очищает таблицы перед тестом.
async fn cleanup(db: &Db) {
    sqlx::query("TRUNCATE commitments, roots, nullifiers RESTART IDENTITY")
        .execute(db.pool())
        .await
        .ok();
}

#[tokio::test]
async fn test_save_deposit_and_get_commitments() {
    let Some(db) = setup().await else { return };
    cleanup(&db).await;

    let pool_address = "test_pool_1";
    let commitment = [1u8; 32];
    let new_root = [2u8; 32];

    db.save_deposit(0, &commitment, &new_root, pool_address, "test_sig_1")
        .await
        .expect("save_deposit failed");

    let commitments = db
        .get_commitments(pool_address)
        .await
        .expect("get_commitments failed");

    assert_eq!(commitments.len(), 1);
    assert_eq!(commitments[0].0, 0);
    assert_eq!(commitments[0].1, commitment.to_vec());
}

#[tokio::test]
async fn test_commitment_exists() {
    let Some(db) = setup().await else { return };
    cleanup(&db).await;

    let pool_address = "test_pool_2";
    let commitment = [3u8; 32];

    assert!(!db.commitment_exists(0).await.unwrap());

    db.save_deposit(0, &commitment, &[0u8; 32], pool_address, "sig")
        .await
        .unwrap();

    assert!(db.commitment_exists(0).await.unwrap());
}

#[tokio::test]
async fn test_save_deposit_idempotent() {
    let Some(db) = setup().await else { return };
    cleanup(&db).await;

    let pool_address = "test_pool_3";
    let commitment = [4u8; 32];
    let root = [5u8; 32];

    // Сохраняем дважды с одинаковым leaf_index
    db.save_deposit(0, &commitment, &root, pool_address, "sig1")
        .await
        .unwrap();
    db.save_deposit(0, &commitment, &root, pool_address, "sig2")
        .await
        .unwrap();

    let commitments = db.get_commitments(pool_address).await.unwrap();
    assert_eq!(
        commitments.len(),
        1,
        "duplicate deposit should not create two rows"
    );
}

#[tokio::test]
async fn test_save_and_check_nullifier() {
    let Some(db) = setup().await else { return };
    cleanup(&db).await;

    let pool_address = "test_pool_4";
    let nullifier = [6u8; 32];

    assert!(!db.is_nullifier_used(&nullifier).await.unwrap());

    db.save_nullifier(&nullifier, pool_address, "recipient_1", 1000, "sig")
        .await
        .unwrap();

    assert!(db.is_nullifier_used(&nullifier).await.unwrap());
}

#[tokio::test]
async fn test_save_nullifier_idempotent() {
    let Some(db) = setup().await else { return };
    cleanup(&db).await;

    let pool_address = "test_pool_5";
    let nullifier = [7u8; 32];

    db.save_nullifier(&nullifier, pool_address, "r1", 100, "sig1")
        .await
        .unwrap();
    db.save_nullifier(&nullifier, pool_address, "r2", 200, "sig2")
        .await
        .unwrap();

    // Второй вызов должен быть проигнорирован (ON CONFLICT DO NOTHING)
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM nullifiers WHERE nullifier_hash = $1")
            .bind(&nullifier[..])
            .fetch_one(db.pool())
            .await
            .unwrap();

    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_get_latest_root() {
    let Some(db) = setup().await else { return };
    cleanup(&db).await;

    let pool_address = "test_pool_6";
    let commitment1 = [8u8; 32];
    let root1 = [9u8; 32];
    let commitment2 = [10u8; 32];
    let root2 = [11u8; 32];

    db.save_deposit(0, &commitment1, &root1, pool_address, "sig1")
        .await
        .unwrap();
    db.save_deposit(1, &commitment2, &root2, pool_address, "sig2")
        .await
        .unwrap();

    let latest = db.get_latest_root(pool_address).await.unwrap();
    assert!(latest.is_some());
    assert_eq!(latest.unwrap(), root2.to_vec());
}

#[tokio::test]
async fn test_get_commitments_ordered_by_leaf_index() {
    let Some(db) = setup().await else { return };
    cleanup(&db).await;

    let pool_address = "test_pool_7";

    // Сохраняем в неправильном порядке
    db.save_deposit(2, &[2u8; 32], &[0u8; 32], pool_address, "sig")
        .await
        .unwrap();
    db.save_deposit(0, &[0u8; 32], &[0u8; 32], pool_address, "sig")
        .await
        .unwrap();
    db.save_deposit(1, &[1u8; 32], &[0u8; 32], pool_address, "sig")
        .await
        .unwrap();

    let commitments = db.get_commitments(pool_address).await.unwrap();

    assert_eq!(commitments.len(), 3);
    assert_eq!(commitments[0].0, 0);
    assert_eq!(commitments[1].0, 1);
    assert_eq!(commitments[2].0, 2);
}
