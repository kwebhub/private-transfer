use super::*;
use crate::{encode_public_inputs, NULLIFIER_SEED, POOL_SEED, VAULT_SEED};
use anchor_lang::solana_program::{instruction::Instruction, program::invoke};

#[derive(Accounts)]
pub struct InitWithdraw<'info> {
    #[account(mut, seeds = [POOL_SEED], bump)]
    pub pool: Account<'info, PoolAcc>,

    /*
     * Аккаунт должен быть изменяемым (mutable),
     * чтобы мы могли добавить нуллификатор после вывода средств
     * сиды PDA — адрес выводится из строки "nullifiers"
     * использует bump-сид, найденный при выводе PDA (Anchor находит его автоматически)
     * аккаунт solana, содержащий данные в форме нашей структуры NullifierRecord,
     * 'info = время жизни транзакции
     */
    // Нуллификатор проверяется через существование PDA
    /// CHECK: Проверяем через PDA
    #[account(mut, seeds = [NULLIFIER_SEED, pool.key().as_ref()], bump)]
    pub nullifier_set: Account<'info, NullifierSetAcc>,

    #[account(mut, seeds = [VAULT_SEED, pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,

    /*
     * /// CHECK: — Anchor требует этот комментарий для каждого UncheckedAccount,
     * чтобы подтвердить, что вы позаботились о безопасности.
     * Без него anchor build выдаст ошибку. Всегда объясняйте, ПОЧЕМУ это безопасно.
     *
     * #[account(mut)] для получателя (recipient): — аккаунт помечен как изменяемый,
     * потому что мы переводим на него SOL.
     * Любой аккаунт, получающий лампорты, должен быть доступен для записи.
     */
    /// CHECK: Validated in instruction logic
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,

    /*
     * внешняя программа, сгенерированная Sunspot:
     * Содержит в себе «зашитый» ключ верификации;
     * Наша программа вызывает её через CPI;
     * Возвращает успех или ошибку в зависимости от валидности доказательства.
     *
     * #[account(constraint = ...)] — это способ Anchor добавлять кастомную валидацию.
     * Ограничение (constraint) выполняется ДО запуска кода инструкции,
     * если проверка не пройдена, то транзакция отменяется.
     * Синтаксис @ PtransError::InvalidVerifier задает кастомную ошибку для вывода.
     *
     * CHECK: Внешняя программа без файла Anchor IDL в нашем проекте.
     * Используем UncheckedAccount,
     * так как не можем указать Program<'info, SunspotVerifier> без импорта типов этой программы.
     * Ограничение гарантирует её валидность.
     */
    /// CHECK: Validated by constraint
    #[account(constraint = verifier_program.key() == VERIFIER_PROGRAM_ID @ PtransError::InvalidVerifier)]
    pub verifier_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

/*
* Клиент поддерживает свою собственную копию дерева Меркла оффчейн:
* - Индексация событий депозита — клиент слушает события DepositEvent.
* - Каждое событие содержит commitment, leaf_index и new_root.
* - Воспроизводя все события по порядку, клиент полностью воссоздает актуальное дерево у себя локально.
*
* Вычисление текущего корня — имея дерево в оперативной памяти,
* клиент последовательно хеширует путь вверх от индекса своего листа (своего обязательства) до тех пор,
* пока не получит текущий корень пула.
*
* Генерация доказательства Меркла (Merkle proof) — это доказательство представляет собой
* список соседних хешей (сестринских узлов),
* необходимых для повторного вычисления корня на основе известного листа.
* Для дерева глубиной 10 этот список состоит ровно из 10 хешей.
*
* При выводе средств программа проверяет ZK-доказательство
* через межпрограммный вызов (CPI) к ончейн-верификатору Sunspot.
* При выводе средств необходимо доказать, что обязательство (commitment) существует в дереве.
* Проверим, что предоставленный корень Меркла совпадает с одним из тех, что уже видела наша программа.
*/
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

    // Prevents front-running by binding proof to recipient
    require!(
        ctx.accounts.recipient.key() == to,
        PtransError::RecipientMismatch
    );
    require!(
        ctx.accounts.pool_vault.lamports() >= amount,
        PtransError::InsufficientVaultBalance
    );

    // Отклоняйте транзакции на вывод средств,
    // если предоставленный корень отсутствует в истории недавних корней пула.
    require!(pool.is_known_root(&root), PtransError::InvalidRoot);

    // Проверяем, использовался ли этот нуллификатор ранее (попытка двойного расходования)
    // Если аккаунт существует → nullifier уже использован
    require!(
        !nullifier_set.contains(&nullifier_hash),
        PtransError::NullifierUsed
    );

    /*
     * === Верификация ZK-доказательства через CPI ===
     * Кодируем публичные входы в формат, который ожидает верификатор
     *
     * вызываем программу-верификатор, передавая доказательство и публичные входы.
     * Если доказательство невалидно, CPI завершается ошибкой,
     * и вся транзакция отменяется — средства не перемещаются.
     */
    let public_inputs = encode_public_inputs(&root, &nullifier_hash, &to, amount);

    // Формируем инструкцию для верификатора
    let verify_instruction = Instruction {
        program_id: VERIFIER_PROGRAM_ID,
        accounts: vec![],
        data: [proof, public_inputs].concat(),
    };

    // Вызываем верификатор через CPI
    invoke(
        &verify_instruction,
        &[ctx.accounts.verifier_program.to_account_info()],
    )
    .map_err(|_| PtransError::InvalidProof)?;

    // Добавляем nullifier в использованные
    nullifier_set.add(nullifier_hash);

    // Получаем bump для подписи vault
    let vault_bump = ctx.bumps.pool_vault;
    let pool_key = pool.key();
    let vault_seeds = &[VAULT_SEED, pool_key.as_ref(), &[vault_bump]];
    let signer_seeds = &[&vault_seeds[..]];

    // Переводим SOL из vault получателю
    let cpi_context = CpiContext::new_with_signer(
        *ctx.accounts.system_program.key,
        system_program::Transfer {
            from: ctx.accounts.pool_vault.to_account_info(),
            to: ctx.accounts.recipient.to_account_info(),
        },
        signer_seeds,
    );
    system_program::transfer(cpi_context, amount)?;

    /*
     * Событие вывода средств должно содержать нуллификатор,
     * чтобы клиенты могли отслеживать, какие именно нуллификаторы уже погашены.
     *
     * Это полезно для исключения отправки заведомо ошибочных транзакций
     * или ведения истории трат конкретного кошелька.
     */
    emit!(WithdrawEvent {
        nullifier_hash,
        recipient: ctx.accounts.recipient.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Withdrawal: {} lamports to {}", amount, to);
    Ok(())
}
