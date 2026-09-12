use super::*;
use crate::{POOL_SEED, VAULT_SEED};

#[derive(Accounts)]
pub struct InitDeposit<'info> {
    #[account(mut, seeds = [POOL_SEED], bump)]
    pub pool: Account<'info, PoolAcc>,

    #[account(mut, seeds = [VAULT_SEED, pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,

    #[account(mut)]
    pub depositor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Client computes commitment and new_root offchain.
/// Invalid roots will cause withdrawal proofs to fail.
pub fn handler_deposit(
    ctx: Context<InitDeposit>,
    commitment: [u8; 32],

    new_root: [u8; 32],
    amount: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;
    require!(pool.next_leaf_index < MAX_LEAVES, PtransError::TreeFull);
    require!(amount >= MIN_DEPOSIT_AMOUNT, PtransError::DepositTooSmall);

    let current_root_index = pool.current_root_index as usize;
    let current_root = pool.roots[current_root_index];
    require!(new_root != current_root, PtransError::InvalidRoot);

    let cpi_context = CpiContext::new(
        *ctx.accounts.system_program.key,
        system_program::Transfer {
            from: ctx.accounts.depositor.to_account_info(),
            to: ctx.accounts.pool_vault.to_account_info(),
        },
    );

    system_program::transfer(cpi_context, amount)?;

    let leaf_index = pool.next_leaf_index;

    pool.next_leaf_index += 1;
    pool.total_deposits += 1;
    emit!(DepositEvent {
        commitment,
        leaf_index,
        timestamp: clock.unix_timestamp,
        new_root,
    });

    msg!(
        "Deposit: {} lamports at leaf index {}, new root: {:?}",
        amount,
        leaf_index,
        new_root
    );
    Ok(())
}
