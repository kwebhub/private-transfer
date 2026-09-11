use std::hash::poseidon2_permutation;

fn hash_2(input: [Field; 2]) -> Field {
    let state = [input[0], input[1], 0, 0];
    poseidon2_permutation(state)[0]
}

fn main(left: Field, right: Field) -> pub Field {
    hash_2([left, right])
}
