use anchor_lang::prelude::*;

#[constant]
pub const POOL_SEED: &[u8] = b"pool";

#[constant]
pub const NULLIFIER_SEED: &[u8] = b"nullifier";

#[constant]
pub const VAULT_SEED: &[u8] = b"vault";

pub const TREE_DEPTH: usize = 20;

pub const MAX_LEAVES: u64 = 1 << TREE_DEPTH;

pub const ROOT_HISTORY_SIZE: usize = 10;

pub const MIN_DEPOSIT_AMOUNT: u64 = 1_000_000; // 0.001 SOL

pub const NR_PUBLIC_INPUTS: u32 = 4;

pub const EMPTY_ROOT: [u8; 32] = [0u8; 32];

pub const VERIFIER_PROGRAM_ID: Pubkey = pubkey!("EewognjaJhZUQgP59BrCx5SaJcsFQ6sn65FEwZ2FdZhg");
