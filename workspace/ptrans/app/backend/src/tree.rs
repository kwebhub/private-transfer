use crate::cache::Cache;
use crate::metrics;
use std::sync::Arc;
use std::time::Instant;

const TREE_DEPTH: u32 = 20;
const EMPTY_LEAF: &str = "0000000000000000000000000000000000000000000000000000000000000000";

async fn poseidon2_hash(
    client: &reqwest::Client,
    merkle_url: &str,
    left: &str,
    right: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let start = Instant::now();
    let url = format!("{}/hash", merkle_url);
    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "left": left, "right": right }))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        metrics::record_tree_error();
        return Err(format!("Merkle /hash error {}: {}", status, body).into());
    }

    let json: serde_json::Value = resp.json().await?;
    let hash = json["hash"]
        .as_str()
        .ok_or("Missing 'hash' field in response")?
        .to_string();

    metrics::observe_tree_hash(start.elapsed().as_secs_f64());

    Ok(hash)
}

pub async fn init_empty_tree(
    cache: &Arc<Cache>,
    merkle_url: &str,
    pool_address: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    cache.set_empty_hash(pool_address, 0, EMPTY_LEAF).await?;
    cache
        .set_tree_level(pool_address, 0, &[EMPTY_LEAF.to_string()])
        .await?;

    let mut current = EMPTY_LEAF.to_string();
    for d in 1..=TREE_DEPTH {
        current = poseidon2_hash(&client, merkle_url, &current, &current).await?;
        cache.set_empty_hash(pool_address, d, &current).await?;
        cache
            .set_tree_level(pool_address, d, &[current.clone()])
            .await?;
    }

    cache.set_tree_root(pool_address, &current).await?;
    cache.set_tree_size(pool_address, 0).await?;

    println!("🌳 Initialized empty tree: root={}", current);
    Ok(())
}

pub async fn add_leaf(
    cache: &Arc<Cache>,
    merkle_url: &str,
    pool_address: &str,
    leaf_index: usize,
    commitment: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let start = Instant::now();
    let client = reqwest::Client::new();

    let size = cache.get_tree_size(pool_address).await?.unwrap_or(0);

    if leaf_index != size {
        return Ok(cache
            .get_tree_root(pool_address)
            .await?
            .unwrap_or_else(|| EMPTY_LEAF.to_string()));
    }

    let mut level0 = cache
        .get_tree_level(pool_address, 0)
        .await?
        .unwrap_or_else(|| vec![EMPTY_LEAF.to_string()]);

    while level0.len() <= leaf_index {
        level0.push(EMPTY_LEAF.to_string());
    }
    level0[leaf_index] = commitment.to_string();

    cache.set_tree_level(pool_address, 0, &level0).await?;

    let mut idx = leaf_index;
    let mut current_level = level0;

    for d in 0..TREE_DEPTH as usize {
        let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };

        let sibling = if sibling_idx < current_level.len() {
            current_level[sibling_idx].clone()
        } else {
            cache
                .get_empty_hash(pool_address, d as u32)
                .await?
                .unwrap_or_else(|| EMPTY_LEAF.to_string())
        };

        let current_hash = current_level[idx].clone();

        let (left, right) = if idx % 2 == 0 {
            (current_hash.clone(), sibling)
        } else {
            (sibling, current_hash.clone())
        };

        let parent = poseidon2_hash(&client, merkle_url, &left, &right).await?;

        let mut next_level: Vec<String> =
            match cache.get_tree_level(pool_address, (d + 1) as u32).await? {
                Some(v) => v,
                None => vec![EMPTY_LEAF.to_string()],
            };

        let parent_idx = idx / 2;
        while next_level.len() <= parent_idx {
            let empty = cache
                .get_empty_hash(pool_address, (d + 1) as u32)
                .await?
                .unwrap_or_else(|| EMPTY_LEAF.to_string());
            next_level.push(empty);
        }
        next_level[parent_idx] = parent.clone();

        cache
            .set_tree_level(pool_address, (d + 1) as u32, &next_level)
            .await?;

        current_level = next_level;
        idx = parent_idx;
    }

    let root = current_level[0].clone();
    cache.set_tree_root(pool_address, &root).await?;
    cache.set_tree_size(pool_address, size + 1).await?;

    metrics::observe_add_leaf(start.elapsed().as_secs_f64());

    println!("🌳 Added leaf {} → new root: {}", leaf_index, root);
    Ok(root)
}

pub async fn get_proof(
    cache: &Arc<Cache>,
    pool_address: &str,
    leaf_index: usize,
) -> Result<(Vec<String>, Vec<bool>, String), Box<dyn std::error::Error + Send + Sync>> {
    let mut proof = Vec::with_capacity(TREE_DEPTH as usize);
    let mut is_even = Vec::with_capacity(TREE_DEPTH as usize);

    let mut idx = leaf_index;

    for d in 0..TREE_DEPTH {
        let level = cache
            .get_tree_level(pool_address, d)
            .await?
            .unwrap_or_else(|| vec![EMPTY_LEAF.to_string()]);

        let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };

        let sibling = if sibling_idx < level.len() {
            level[sibling_idx].clone()
        } else {
            cache
                .get_empty_hash(pool_address, d)
                .await?
                .unwrap_or_else(|| EMPTY_LEAF.to_string())
        };

        proof.push(sibling);
        is_even.push(idx % 2 == 0);
        idx /= 2;
    }

    let root = cache
        .get_tree_root(pool_address)
        .await?
        .unwrap_or_else(|| EMPTY_LEAF.to_string());

    Ok((proof, is_even, root))
}
