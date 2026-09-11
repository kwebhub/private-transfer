use ::std::hash::poseidon2_permutation;

pub fn compute_merkle_root<let DEPTH: u32>(
    leaf: Field,
    path: [Field; DEPTH],
    is_even: [bool; DEPTH],
) -> Field {
    let mut current = leaf;

    for i in 0..DEPTH {
        let sibling = path[i];

        let (left, right) = if is_even[i] {
            (current, sibling)
        } else {
            (sibling, current)
        };

        let state = [left, right, 0, 0];
        current = poseidon2_permutation(state)[0];
    }

    current
}
pub fn verify_merkle_proof<let DEPTH: u32>(
    leaf: Field,
    root: Field,
    path: [Field; DEPTH],
    is_even: [bool; DEPTH],
) -> bool {
    let computed_root = compute_merkle_root(leaf, path, is_even);

    computed_root == root
}
