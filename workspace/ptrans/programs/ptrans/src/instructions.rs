pub mod deposit;
pub mod pool;
pub mod withdraw;

use crate::{constants::*, error::*, events::*, state::*};
use anchor_lang::{prelude::*, system_program};

pub use deposit::*;
pub use pool::*;
pub use withdraw::*;

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
    inputs.extend_from_slice(recipient.as_ref());

    let mut amount_bytes = [0u8; 32];
    amount_bytes[24..32].copy_from_slice(&amount.to_be_bytes());
    inputs.extend_from_slice(&amount_bytes);

    inputs
}
