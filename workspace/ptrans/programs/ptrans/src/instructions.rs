pub mod deposit;
pub mod pool;
pub mod withdraw;

use crate::{constants::*, error::*, events::*, state::*};
use anchor_lang::{prelude::*, system_program};

pub use deposit::*;
pub use pool::*;
pub use withdraw::*;

/*
* Верификатор Sunspot ожидает определенный бинарный формат входных данных.
* Нам необходимо закодировать наши публичные входы (корень, нуллификатор, получатель, сумма)
* в точности так, как этого требует верификатор.
*
* Формат данных инструкции выглядит так:
* proof_bytes || public_witness_bytes (байты доказательства || байты публичных свидетелей)
*/

/*
* Эта функция кодирует публичные входы в формат, ожидаемый верификатором Gnark/Sunspot.
* Верификатор ожидает бинарный формат:
* 12-байтовый заголовок,
* за которым следует каждый публичный вход в виде 32-байтового элемента поля в формате big-endian.
*
* Big-endian означает, что сначала идет самый значимый байт — противоположность тому,
* как числа хранятся в Solana и большинстве процессоров (little-endian).
*/
/// Gnark witness format: 12-byte header + 4x32-byte public inputs
pub fn encode_public_inputs(
    root: &[u8; 32],
    nullifier_hash: &[u8; 32],
    recipient: &Pubkey,
    amount: u64,
) -> Vec<u8> {
    /*
     * Выделяем память заранее: 12 байт заголовка + 4 входа × 32 байта каждый = 140 байт
     * Мы пишем 12 + 128 вместо 140, чтобы наглядно показать, ОТКУДА берется это число.
     * Vec::with_capacity заранее резервирует память,
     * чтобы избежать изменения размеров вектора при вызовах extend_from_slice.
     */
    let mut inputs = Vec::with_capacity(12 + 128);

    /*
     * === Заголовок Gnark (12 байт) ===
     *
     * Верификатор Gnark ожидает:
     * Байты 0-3: Количество публичных входов (big-endian u32)
     * Байты 4-7: Количество обязательств (commitments), для нас всегда 0 (big-endian u32)
     *
     * Gnark поддерживает схемы "обязательств",
     * но мы их не используем — это отличается от наших депозитных обязательств!
     *
     * Байты 8-11: Снова количество публичных входов (big-endian u32) — это особенность формата Gnark
     *
     * extend_from_slice добавляет срез байтов к нашему Vec — эффективный способ сборки байтовых массивов
     */

    // Header: num_public (4) | num_private (4) | vector_len (4)
    inputs.extend_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());
    inputs.extend_from_slice(&0u32.to_be_bytes());
    inputs.extend_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());

    /* === Публичные входы (каждый по 32 байта, big-endian) ===
     * ВАЖНО: Порядок должен в точности соответствовать объявлению публичных входов в ZK-схеме!
     * Наша схема объявляет: root, nullifier_hash, recipient, amount.
     *
     * Корень Меркла (Merkle root) — доказывает, что обязательство существует в дереве.
     * Хеш нуллификатора (Nullifier hash) — предотвращает двойное расходование средств.
     * Публичный ключ получателя (Recipient pubkey) — кто получает средства (32 байта).
     *
     * Метод as_ref() преобразует Pubkey в &[u8] — это необходимо,
     * так как Pubkey не является напрямую байтовым массивом.
     * root и nullifier_hash уже имеют тип [u8; 32], поэтому им преобразование не требуется.
     *
     * Сумма (Amount) — дополняется нулями до 32 байт (тип u64 занимает всего 8 байт).
     * Заполняем левую часть 24 нулевыми байтами,
     * а затем записываем 8-байтовое значение в формате big-endian.
     */
    inputs.extend_from_slice(root);
    inputs.extend_from_slice(nullifier_hash);
    inputs.extend_from_slice(recipient.as_ref());

    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    inputs.extend_from_slice(&amount_bytes);

    inputs
}
