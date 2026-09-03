pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use events::*;
pub use instructions::*;
pub use state::*;

declare_id!("FbXJSZ171dcnHJVrd5E6KwvXAx7bMgxC44McF84vJ6cK");

#[program]
pub mod ptrans {
    use super::*;

    pub fn pool(ctx: Context<InitPool>) -> Result<()> {
        handler_pool(ctx)
    }

    pub fn deposit(
        ctx: Context<InitDeposit>,
        commitment: [u8; 32],
        new_root: [u8; 32],
        amount: u64,
    ) -> Result<()> {
        handler_deposit(ctx, commitment, new_root, amount)
    }

    pub fn withdraw(
        ctx: Context<InitWithdraw>,
        proof: Vec<u8>,
        nullifier_hash: [u8; 32],
        root: [u8; 32],
        to: Pubkey,
        amount: u64,
    ) -> Result<()> {
        handler_withdraw(ctx, proof, nullifier_hash, root, to, amount)
    }
}
