use std::hash::poseidon2_permutation;

fn hash_1(input: [Field; 1]) -> Field {
    let state = [input[0], 0, 0, 0];
    poseidon2_permutation(state)[0]
}

fn hash_3(input: [Field; 3]) -> Field {
    let state = [input[0], input[1], input[2], 0];
    poseidon2_permutation(state)[0]
}

fn main(nullifier: Field, secret: Field, amount: Field) -> pub (Field, Field) {
    let commitment = hash_3([nullifier, secret, amount]);
    let nullifier_hash = hash_1([nullifier]);
    (commitment, nullifier_hash)
}
