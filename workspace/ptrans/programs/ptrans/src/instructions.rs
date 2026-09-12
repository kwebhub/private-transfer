pub mod deposit;
pub mod pool;
pub mod withdraw;

use crate::{constants::*, error::*, events::*, state::*};
use anchor_lang::{prelude::*, system_program};

pub use deposit::*;
pub use pool::*;
pub use withdraw::*;

/// Модуль поля BN254 (scalar field), big-endian
const BN254_FR_MODULUS: [u8; 32] = [
    0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58, 0x5d,
    0x97, 0x81, 0x6a, 0x91, 0x68, 0x71, 0xca, 0x8d, 0x3c, 0x20, 0x8c, 0x16, 0xd8, 0x7c, 0xfd, 0x47,
];

/// Вычитает b из a как 256-битные big-endian числа
fn sub_256(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut borrow: i16 = 0;
    for i in (0..32).rev() {
        let diff = a[i] as i16 - b[i] as i16 - borrow;
        if diff < 0 {
            result[i] = (diff + 256) as u8;
            borrow = 1;
        } else {
            result[i] = diff as u8;
            borrow = 0;
        }
    }
    result
}

/// Приводит 32 байта (big-endian) к модулю BN254
fn reduce_to_field(bytes: &[u8; 32]) -> [u8; 32] {
    // Если число меньше модуля — возвращаем как есть
    if bytes < &BN254_FR_MODULUS {
        return *bytes;
    }

    // Иначе — вычитаем модуль, пока не станет меньше
    // (входное число < 2^256, модуль ~2^254, значит хватит 4 итераций)
    let mut result = *bytes;
    for _ in 0..4 {
        if result < BN254_FR_MODULUS {
            break;
        }
        result = sub_256(&result, &BN254_FR_MODULUS);
    }
    result
}

/// Gnark witness format: 12-byte header + 4x32-byte public inputs
pub fn encode_public_inputs(
    root: &[u8; 32],
    nullifier_hash: &[u8; 32],
    recipient: &Pubkey,
    amount: u64,
) -> Vec<u8> {
    let mut inputs = Vec::with_capacity(12 + 128);

    inputs.extend_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());
    inputs.extend_from_slice(&0u32.to_be_bytes());
    inputs.extend_from_slice(&NR_PUBLIC_INPUTS.to_be_bytes());

    inputs.extend_from_slice(root);
    inputs.extend_from_slice(nullifier_hash);
    let recipient_bytes: &[u8; 32] = recipient.as_ref().try_into().unwrap();
    inputs.extend_from_slice(&reduce_to_field(recipient_bytes));

    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    inputs.extend_from_slice(&amount_bytes);

    inputs
}
