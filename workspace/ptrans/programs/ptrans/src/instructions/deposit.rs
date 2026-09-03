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

/*
* При каждом депозите необходимо:
* - Принимать новый корень Меркла, вычисленный оффчейн клиентом после добавления обязательства в дерево.
* - Сохранять этот корень в нашу историю.
* - Логировать индекс листа (emit event), чтобы клиенты могли строить доказательства членства Меркла (Merkle proofs). Технически, это необязательно, если мы логируем каждое событие с обязательством, но это хорошая практика: так проще искать конкретные обязательства или восстанавливать состояние при пропусках событий.
*
* Вместо записи данных о пользователе мы сохраняем обязательство (commitment),
* а затем человек должен будет доказать, что он знает секрет этого обязательства.
*
* Обязательство (commitment) представляет собой хеш,
* который на выходе дает 256 бит (32 байта × 8 бит = 256 бит).
* Бэкенд вычисляет этот хеш, преобразует его в байты и отправляет в программу.
*/
/// Client computes commitment and new_root offchain.
/// Invalid roots will cause withdrawal proofs to fail.
pub fn handler_deposit(
    ctx: Context<InitDeposit>,
    commitment: [u8; 32],

    // Дерево Меркла поддерживается клиентом в оффчейне.
    // После добавления обязательства в качестве листа, клиент вычисляет
    // новый корень и передает его сюда.
    // Программа просто сохраняет его.
    new_root: [u8; 32],
    amount: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;
    /*
     * Мы не можем принимать новые депозиты, если все 1024 позиции листьев уже заняты.
     * Вставить проверку перед вызовом CPI:
     */
    require!(pool.next_leaf_index < MAX_LEAVES, PtransError::TreeFull);
    require!(amount >= MIN_DEPOSIT_AMOUNT, PtransError::DepositTooSmall);

    // Проверяем, что новый корень действительно новый (не равен последнему)
    let current_root_index = pool.current_root_index as usize;
    let current_root = pool.roots[current_root_index];
    require!(new_root != current_root, PtransError::InvalidRoot);

    // Transfer SOL to vault
    let cpi_context = CpiContext::new(
        *ctx.accounts.system_program.key,
        system_program::Transfer {
            from: ctx.accounts.depositor.to_account_info(),
            to: ctx.accounts.pool_vault.to_account_info(),
        },
    );

    // После успешного выполнения перевода SOL мы обновляем состояние дерева и генерируем событие (event).
    system_program::transfer(cpi_context, amount)?;

    // Сохраняем позицию листа, в которую было добавлено это обязательство
    let leaf_index = pool.next_leaf_index;

    // Сдвигаем индекс на следующую позицию для будущего депозита
    pool.next_leaf_index += 1;
    pool.total_deposits += 1;
    /*
     * Структура нашего пула хранит корень дерева Меркла (Merkle Tree root).
     * Нам не нужно хранить все дерево целиком на блокчейне — его можно поддерживать оффчейн.
     * Поэтому при каждом депозите мы генерируем ончейн-события (events) с обязательством (commitment):
     * это позволяет индексаторам вести полную копию дерева Меркла.
     * В программе нужно лишь отслеживать текущую позицию в дереве и актуальный корень Меркла.
     */
    emit!(DepositEvent {
        commitment,
        // Чтобы клиент знал, где именно в дереве находится это обязательство
        leaf_index,
        timestamp: clock.unix_timestamp,
        // Чтобы другие клиенты могли обновить локальную копию дерева
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
