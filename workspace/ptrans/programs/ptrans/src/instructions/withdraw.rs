use super::*;
use crate::{encode_public_inputs, NULLIFIER_SEED, POOL_SEED, VAULT_SEED};
use anchor_lang::solana_program::{instruction::Instruction, program::invoke};

#[derive(Accounts)]
pub struct InitWithdraw<'info> {
    #[account(mut, seeds = [POOL_SEED], bump)]
    pub pool: Account<'info, PoolAcc>,

    /// CHECK: Проверяем через PDA
    #[account(mut, seeds = [NULLIFIER_SEED, pool.key().as_ref()], bump)]
    pub nullifier_set: Account<'info, NullifierSetAcc>,

    #[account(mut, seeds = [VAULT_SEED, pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,

    /// CHECK: Validated in instruction logic
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: Validated by constraint
    #[account(constraint = verifier_program.key() == VERIFIER_PROGRAM_ID @ PtransError::InvalidVerifier)]
    pub verifier_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler_withdraw(
    ctx: Context<InitWithdraw>,

    // ZK-доказательство, сгенерированное клиентом (324 байта)
    proof: Vec<u8>,
    nullifier_hash: [u8; 32],
    root: [u8; 32],
    to: Pubkey,
    amount: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let nullifier_set = &mut ctx.accounts.nullifier_set;
    let clock = Clock::get()?;

    require!(
        ctx.accounts.recipient.key() == to,
        PtransError::RecipientMismatch
    );
    require!(
        ctx.accounts.pool_vault.lamports() >= amount,
        PtransError::InsufficientVaultBalance
    );

    require!(pool.is_known_root(&root), PtransError::InvalidRoot);

    require!(
        !nullifier_set.contains(&nullifier_hash),
        PtransError::NullifierUsed
    );

    let public_inputs = encode_public_inputs(&root, &nullifier_hash, &to, amount);

    let verify_instruction = Instruction {
        program_id: VERIFIER_PROGRAM_ID,
        accounts: vec![],
        data: [proof, public_inputs].concat(),
    };

    invoke(
        &verify_instruction,
        &[ctx.accounts.verifier_program.to_account_info()],
    )
    .map_err(|_| PtransError::InvalidProof)?;

    nullifier_set.add(nullifier_hash);

    let vault_bump = ctx.bumps.pool_vault;
    let pool_key = pool.key();
    let vault_seeds = &[VAULT_SEED, pool_key.as_ref(), &[vault_bump]];
    let signer_seeds = &[&vault_seeds[..]];

    let cpi_context = CpiContext::new_with_signer(
        *ctx.accounts.system_program.key,
        system_program::Transfer {
            from: ctx.accounts.pool_vault.to_account_info(),
            to: ctx.accounts.recipient.to_account_info(),
        },
        signer_seeds,
    );
    system_program::transfer(cpi_context, amount)?;

    emit!(WithdrawEvent {
        nullifier_hash,
        recipient: ctx.accounts.recipient.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Withdrawal: {} lamports to {}", amount, to);
    Ok(())
}
