use super::*;
use crate::{EMPTY_ROOT, NULLIFIER_SEED, POOL_SEED, VAULT_SEED};

#[derive(Accounts)]
pub struct InitPool<'info> {
    // PDA, состояние приватного пула:
    // история корней Меркла, индекс листьев, счетчик депозитов
    #[account(
        init,
        payer = authority,
        space = PoolAcc::DISCRIMINATOR.len() + PoolAcc::INIT_SPACE,
        seeds = [POOL_SEED],
        bump
    )]
    pub pool: Account<'info, PoolAcc>,

    // PDA, хранит хеши всех использованных нуллификаторов для указаннго пула
    // и отслеживает повторное использованные хешей нуллификаторов
    #[account(
        init,
        payer = authority,
        space = NullifierSetAcc::DISCRIMINATOR.len() + NullifierSetAcc::INIT_SPACE,
        seeds = [NULLIFIER_SEED, pool.key().as_ref()],
        bump
    )]
    pub nullifier_set: Account<'info, NullifierSetAcc>,

    /// CHECK: Vault для хранения SOL
    // PDA, удерживает реальные SOL
    #[account(mut, seeds = [VAULT_SEED, pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler_pool(ctx: Context<InitPool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.authority = ctx.accounts.authority.key();
    pool.next_leaf_index = 0;
    pool.total_deposits = 0;
    pool.current_root_index = 0;
    pool.roots[0] = EMPTY_ROOT;

    // Инициализируем все остальные корни тем же значением
    for i in 1..ROOT_HISTORY_SIZE {
        pool.roots[i] = EMPTY_ROOT;
    }

    let nullifier_set = &mut ctx.accounts.nullifier_set;
    nullifier_set.pool = pool.key();
    nullifier_set.nullifiers = Vec::new();

    msg!("Pool initialized with admin: {}", pool.authority);
    Ok(())
}
