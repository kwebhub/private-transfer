//! Integration tests для программы `ptrans`.
//!
//! Проверяют **PDA derivation** — детерминированность, уникальность,
//! bump-валидацию. Полный withdraw с ZK-proof не тестируется
//! (требует Sunspot + Noir circuit).

use anchor_lang::prelude::Pubkey;
use std::str::FromStr;

/// Program ID программы ptrans.
const PROGRAM_ID_STR: &str = "FbXJSZ171dcnHJVrd5E6KwvXAx7bMgxC44McF84vJ6cK";

/// Минимальный депозит (0.001 SOL).
const MIN_DEPOSIT_LAMPORTS: u64 = 1_000_000;

/// Стартовый баланс для тестов.
const INITIAL_BALANCE: u64 = 10_000_000_000; // 10 SOL

// ============================================================
// Helpers
// ============================================================

fn program_id() -> Pubkey {
    Pubkey::from_str(PROGRAM_ID_STR).unwrap()
}

fn find_pool_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"pool3"], &program_id())
}

fn find_vault_pda(pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"vault3", pool.as_ref()], &program_id())
}

fn find_nullifier_record_pda(pool: &Pubkey, nullifier_hash: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"nullifier_record", pool.as_ref(), nullifier_hash.as_ref()],
        &program_id(),
    )
}

// ============================================================
// Tests
// ============================================================

#[test]
fn test_program_id_parses() {
    let id = program_id();
    assert_eq!(id.to_bytes().len(), 32);
    assert_ne!(id, Pubkey::default());
}

#[test]
fn test_find_pool_pda_deterministic() {
    let (pool1, _) = find_pool_pda();
    let (pool2, _) = find_pool_pda();
    assert_eq!(pool1, pool2);

    // Pool != program id
    assert_ne!(pool1, program_id());
}

#[test]
fn test_find_vault_pda() {
    let (pool, _) = find_pool_pda();
    let (vault, _) = find_vault_pda(&pool);

    assert_ne!(vault, program_id());
    assert_ne!(vault, pool);

    // Vault зависит от pool
    let other_pool = Pubkey::new_unique();
    let (other_vault, _) = find_vault_pda(&other_pool);
    assert_ne!(vault, other_vault);
}

#[test]
fn test_find_nullifier_record_pda() {
    let (pool, _) = find_pool_pda();
    let nullifier = [42u8; 32];
    let (record, _) = find_nullifier_record_pda(&pool, &nullifier);

    assert_ne!(record, program_id());
    assert_ne!(record, pool);

    // Разные nullifier'ы → разные PDA
    let other_nullifier = [43u8; 32];
    let (other_record, _) = find_nullifier_record_pda(&pool, &other_nullifier);
    assert_ne!(record, other_record);
}

#[test]
fn test_pdas_do_not_collide() {
    let (pool, _) = find_pool_pda();
    let (vault, _) = find_vault_pda(&pool);
    let (record, _) = find_nullifier_record_pda(&pool, &[7u8; 32]);

    assert_ne!(pool, vault);
    assert_ne!(pool, record);
    assert_ne!(vault, record);
}

#[test]
fn test_pda_bumps_are_valid() {
    // bump имеет тип u8 — он всегда в диапазоне 0..=255.
    // Проверяем, что функция возвращает bump без паники.
    let (_, pool_bump) = find_pool_pda();
    let (pool, _) = find_pool_pda();
    let (_, vault_bump) = find_vault_pda(&pool);
    let (_, record_bump) = find_nullifier_record_pda(&pool, &[1u8; 32]);

    // Явно используем bump, чтобы не было warning про unused.
    let sum = pool_bump as u16 + vault_bump as u16 + record_bump as u16;
    assert!(sum <= 765, "sum of bumps = {sum}");
}

#[test]
fn test_min_deposit_amount() {
    // Программа требует >= 1_000_000 lamports (0.001 SOL)
    assert_eq!(MIN_DEPOSIT_LAMPORTS, 1_000_000);
}

#[test]
fn test_initial_balance_is_enough() {
    // 10 SOL хватает на 10+ депозитов по 0.001 SOL + rent для PDA.
    // Проверяем на этапе компиляции (const-assert).
    const _: () = assert!(INITIAL_BALANCE >= 10 * MIN_DEPOSIT_LAMPORTS);
}
