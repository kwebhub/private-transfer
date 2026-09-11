mod merkle_tree;
use merkle_tree::compute_merkle_root;
use std::hash::poseidon2_permutation;

global TREE_DEPTH: u32 = 20;

fn hash_1(input: [Field; 1]) -> Field {
    let state = [input[0], 0, 0, 0];
    poseidon2_permutation(state)[0]
}

fn hash_2(input: [Field; 2]) -> Field {
    let state = [input[0], input[1], 0, 0];
    poseidon2_permutation(state)[0]
}

fn hash_3(input: [Field; 3]) -> Field {
    let state = [input[0], input[1], input[2], 0];
    poseidon2_permutation(state)[0]
}

fn main(
    root: pub Field,
    nullifier_hash: pub Field,
    recipient: pub Field,
    amount: pub Field,
    nullifier: Field,
    secret: Field,
    merkle_proof: [Field; TREE_DEPTH],
    is_even: [bool; TREE_DEPTH],
) {
    let commitment = hash_3([nullifier, secret, amount]);

    let computed_nullifier_hash = hash_1([nullifier]);

    assert(computed_nullifier_hash == nullifier_hash, "Invalid nullifier hash");

    let computed_root = compute_merkle_root(commitment, merkle_proof, is_even);

    assert(computed_root == root, "Invalid Merkle proof");

    let _ = recipient;
}

fn compute_commitment(nullifier: Field, secret: Field, amount: Field) -> Field {
    hash_3([nullifier, secret, amount])
}

fn compute_nullifier_hash(nullifier: Field) -> Field {
    hash_1([nullifier])
}

fn compute_empty_tree_zeros() -> [Field; TREE_DEPTH] {
    let mut zeros: [Field; TREE_DEPTH] = [0; TREE_DEPTH];
    let mut current: Field = 0;

    for i in 0..TREE_DEPTH {
        zeros[i] = current;
        current = hash_2([current, current]);
    }

    zeros
}

#[test]
fn test_commitment_computation() {
    let nullifier = 12345;
    let secret = 67890;
    let amount = 1000000000; // 1 SOL in lamports
    let commitment = compute_commitment(nullifier, secret, amount);
    let commitment2 = compute_commitment(nullifier, secret, amount);
    assert(commitment == commitment2);

    let commitment3 = compute_commitment(nullifier, secret, amount + 1);
    assert(commitment != commitment3);
}

#[test]
fn test_nullifier_hash() {
    let nullifier = 12345;
    let hash1 = compute_nullifier_hash(nullifier);
    let hash2 = compute_nullifier_hash(nullifier);
    assert(hash1 == hash2);

    let hash3 = compute_nullifier_hash(nullifier + 1);
    assert(hash1 != hash3);
}

#[test]
fn test_commitment_uniqueness() {
    let nullifier = 99999;
    let secret = 88888;
    let amount1 = 100000000; // 0.1 SOL
    let amount2 = 200000000; // 0.2 SOL

    let commitment1 = compute_commitment(nullifier, secret, amount1);
    let commitment2 = compute_commitment(nullifier, secret, amount2);

    assert(commitment1 != commitment2);
}

#[test]
fn test_generate_valid_inputs() {
    let nullifier: Field = 12345;
    let secret: Field = 67890;
    let amount: Field = 1000000000;
    let recipient: Field = 123456789;

    let commitment = compute_commitment(nullifier, secret, amount);
    let nullifier_hash = compute_nullifier_hash(nullifier);
    let zeros = compute_empty_tree_zeros();

    let merkle_proof = zeros;
    let is_even = [true; TREE_DEPTH];
    let root = merkle_tree::compute_merkle_root(commitment, merkle_proof, is_even);

    println("=== Данные для Prover.toml ===");
    println(f"root = \"{root}\"");
    println(f"secret = \"{secret}\"");
    println(f"amount = \"{amount}\"");

    println("is_even = [");
    for i in 0..TREE_DEPTH {
        print("1");
        if i < TREE_DEPTH - 1 {
            println(",");
        } else {
            println("");
        }
    }
    println("]");

    println("merkle_proof = [");
    for i in 0..TREE_DEPTH {
        let prove = zeros[i];
        print(f"\"{prove}\"");
        if i < TREE_DEPTH - 1 {
            println(",");
        } else {
            println("");
        }
    }
    println("]");

    println(f"nullifier = \"{nullifier}\"");
    println(f"nullifier_hash = \"{nullifier_hash}\"");
    println(f"recipient = \"{recipient}\"");
    println("=== Конец данных ===");

    let computed = merkle_tree::compute_merkle_root(commitment, merkle_proof, is_even);

    assert(computed == root);
}

#[test]
fn test_print_commitment_for_js() {
    let nullifier: Field = 12345;
    let secret: Field = 67890;
    let amount: Field = 1000000000;

    let commitment = compute_commitment(nullifier, secret, amount);
    let nullifier_hash = compute_nullifier_hash(nullifier);

    println(f"commitment = 0x{commitment}");
    println(f"nullifier_hash = 0x{nullifier_hash}");
}
