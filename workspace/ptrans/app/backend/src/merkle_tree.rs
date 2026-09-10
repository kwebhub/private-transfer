use ark_bn254::Fr;
use ark_ff::{BigInteger, PrimeField};
use poseidon_rs::Poseidon;
use std::collections::HashMap;

pub const TREE_DEPTH: usize = 20;
pub const EMPTY_ROOT: [u8; 32] = [0u8; 32];

pub struct Pool {
    pub commitments: Vec<[u8; 32]>,
    pub tree: Vec<Vec<[u8; 32]>>,
    pub root: [u8; 32],
    pub poseidon: Poseidon,
}

impl Pool {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let poseidon = Poseidon::new();
        let mut pool = Self {
            commitments: Vec::new(),
            tree: Vec::new(),
            root: EMPTY_ROOT,
            poseidon,
        };
        pool.init_tree().await?;
        Ok(pool)
    }

    async fn init_tree(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let empty_leaf = [0u8; 32];
        let leaves = vec![empty_leaf; 1 << TREE_DEPTH];
        self.tree = self.build_tree(leaves).await?;
        self.root = *self.tree.last().unwrap().first().unwrap();
        Ok(())
    }

    async fn build_tree(
        &self,
        mut leaves: Vec<[u8; 32]>,
    ) -> Result<Vec<Vec<[u8; 32]>>, Box<dyn std::error::Error>> {
        let mut tree = vec![leaves];
        for _ in 0..TREE_DEPTH {
            let current_level = tree.last().unwrap();
            let mut next_level = Vec::with_capacity((current_level.len() + 1) / 2);
            for chunk in current_level.chunks(2) {
                let left = &chunk[0];
                let right = chunk.get(1).unwrap_or(&[0u8; 32]);
                let hash = self.hash_pair(left, right);
                next_level.push(hash);
            }
            tree.push(next_level);
        }
        Ok(tree)
    }

    pub async fn add_commitment(
        &mut self,
        commitment: [u8; 32],
    ) -> Result<(u64, [u8; 32]), Box<dyn std::error::Error>> {
        let leaf_index = self.commitments.len() as u64;
        if leaf_index >= (1 << TREE_DEPTH) {
            return Err("Tree is full".into());
        }

        self.commitments.push(commitment);
        self.rebuild_tree().await?;
        Ok((leaf_index, self.root))
    }

    async fn rebuild_tree(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut leaves = vec![[0u8; 32]; 1 << TREE_DEPTH];
        for (i, commitment) in self.commitments.iter().enumerate() {
            leaves[i] = *commitment;
        }
        self.tree = self.build_tree(leaves).await?;
        self.root = *self.tree.last().unwrap().first().unwrap();
        Ok(())
    }

    pub async fn get_merkle_proof(
        &self,
        leaf_index: u64,
    ) -> Option<(Vec<[u8; 32]>, Vec<bool>, [u8; 32])> {
        if leaf_index >= self.commitments.len() as u64 {
            return None;
        }

        let mut proof = Vec::with_capacity(TREE_DEPTH);
        let mut is_even = Vec::with_capacity(TREE_DEPTH);
        let mut idx = leaf_index;

        for level in 0..TREE_DEPTH {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            let sibling = self.tree[level][sibling_idx as usize];
            proof.push(sibling);
            is_even.push(idx % 2 == 0);
            idx /= 2;
        }

        Some((proof, is_even, self.root))
    }

    pub async fn get_root(&self) -> ([u8; 32], u64) {
        (self.root, self.commitments.len() as u64)
    }

    fn hash_pair(&self, left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let left_fr = Fr::from_be_bytes_mod_order(left);
        let right_fr = Fr::from_be_bytes_mod_order(right);
        let hash = self.poseidon.hash(vec![left_fr, right_fr]);
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash.into_repr().to_bytes_be());
        result
    }
}
