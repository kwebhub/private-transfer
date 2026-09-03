use crate::ROOT_HISTORY_SIZE;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PoolAcc {
    pub authority: Pubkey,
    pub next_leaf_index: u64,
    pub total_deposits: u64,
    pub current_root_index: u64,
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE],
}

impl PoolAcc {
    pub fn is_known_root(&self, root: &[u8; 32]) -> bool {
        self.roots.iter().any(|r| r == root)
    }
    pub fn add_root(&mut self, new_root: [u8; 32]) {
        let index = ((self.current_root_index + 1) % ROOT_HISTORY_SIZE as u64) as usize;
        self.current_root_index = index as u64;
        self.roots[index] = new_root;
    }
}

#[account]
pub struct NullifierRecord {
    pub pool: Pubkey,
    pub nullifier: [u8; 32],
    pub used: bool,
}

impl NullifierRecord {
    pub const DISCRIMINATOR: &'static [u8] = &[0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF];
    pub const INIT_SPACE: usize = 32 + 32 + 1;
}

#[account]
#[derive(InitSpace)]
pub struct NullifierSetAcc {
    pub pool: Pubkey,
    #[max_len(1024)]
    pub nullifiers: Vec<[u8; 32]>,
}

impl NullifierSetAcc {
    pub const DISCRIMINATOR: &'static [u8] = &[0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x90];
    pub const INIT_SPACE: usize = 32 + 4; // pool + vec capacity

    pub fn contains(&self, nullifier: &[u8; 32]) -> bool {
        self.nullifiers.contains(nullifier)
    }

    pub fn add(&mut self, nullifier: [u8; 32]) {
        self.nullifiers.push(nullifier);
    }
}
