use super::*;
use crate::mock::MockStateNodeStore;
use anyhow::Result;
use forkable_jellyfish_merkle::node_type::Node;
use starcoin_crypto::hash::*;
use std::sync::Arc;

#[test]
pub fn test_put_blob() -> Result<()> {
    let s = MockStateNodeStore::new();
    let state = StateTree::new(Arc::new(s), None);
    assert_eq!(state.root_hash(), HashValue::zero());

    let hash_value = HashValue::random();

    let account1 = update_nibble(&hash_value, 0, 1);
    let account1 = update_nibble(&account1, 2, 2);
    let new_root_hash = state.put_blob_set(vec![(account1, vec![0, 0, 0])])?;
    assert_eq!(state.root_hash(), new_root_hash);
    assert_eq!(state.get(&account1)?, Some(vec![0, 0, 0]));
    assert_eq!(state.get(&update_nibble(&hash_value, 0, 8))?, None);
    let (root, updates) = state.change_sets();
    assert_eq!(root, new_root_hash);
    assert_eq!(updates.num_stale_leaves, 0);
    assert_eq!(updates.num_new_leaves, 1);
    assert_eq!(updates.node_batch.len(), 1);
    assert_eq!(updates.stale_node_index_batch.len(), 1);

    let account2 = update_nibble(&account1, 0, 2);
    let new_root_hash = state.put_blob_set(vec![(account2, vec![0, 0, 0])])?;
    assert_eq!(state.root_hash(), new_root_hash);
    assert_eq!(state.get(&account2)?, Some(vec![0, 0, 0]));
    let (root, updates) = state.change_sets();
    assert_eq!(root, new_root_hash);
    assert_eq!(updates.num_stale_leaves, 0);
    assert_eq!(updates.num_new_leaves, 2);
    assert_eq!(updates.node_batch.len(), 3);
    assert_eq!(updates.stale_node_index_batch.len(), 1);

    // modify existed account
    let new_root_hash = state.put_blob_set(vec![(account1, vec![1, 1, 1])])?;
    assert_eq!(state.root_hash(), new_root_hash);
    assert_eq!(state.get(&account1)?, Some(vec![1, 1, 1]));
    let (root, updates) = state.change_sets();
    assert_eq!(root, new_root_hash);
    assert_eq!(updates.num_stale_leaves, 0);
    assert_eq!(updates.num_new_leaves, 2);
    assert_eq!(updates.node_batch.len(), 3);
    assert_eq!(updates.stale_node_index_batch.len(), 1);

    let account3 = update_nibble(&account1, 2, 3);
    let new_root_hash =
        state.put_blob_set(vec![(account1, vec![1, 1, 0]), (account3, vec![0, 0, 0])])?;
    assert_eq!(state.root_hash(), new_root_hash);
    assert_eq!(state.get(&account1)?, Some(vec![1, 1, 0]));
    assert_eq!(state.get(&account2)?, Some(vec![0, 0, 0]));
    assert_eq!(state.get(&account3)?, Some(vec![0, 0, 0]));

    let (_, updates) = state.change_sets();
    assert_eq!(updates.num_stale_leaves, 0);
    assert_eq!(updates.num_new_leaves, 3);
    assert_eq!(updates.node_batch.len(), 6);
    assert_eq!(updates.stale_node_index_batch.len(), 1);
    Ok(())
}

#[test]
pub fn test_state_commit() -> Result<()> {
    // TODO: once storage support batch put, finish this.
    let s = MockStateNodeStore::new();
    let state = StateTree::new(Arc::new(s), None);
    assert_eq!(state.root_hash(), HashValue::zero());

    let hash_value = HashValue::random();

    let account1 = update_nibble(&hash_value, 0, 1);
    let account1 = update_nibble(&account1, 2, 2);
    let new_root_hash = state.put_blob_set(vec![(account1, vec![0, 0, 0])])?;

    let account3 = update_nibble(&account1, 2, 3);
    let new_root_hash =
        state.put_blob_set(vec![(account1, vec![1, 1, 0]), (account3, vec![0, 0, 0])])?;

    state.commit()?;
    assert_eq!(state.root_hash(), new_root_hash);
    assert_eq!(state.get(&account1)?, Some(vec![1, 1, 0]));
    assert_eq!(state.get(&account3)?, Some(vec![0, 0, 0]));
    assert_eq!(state.get(&update_nibble(&account1, 2, 10))?, None);
    Ok(())
}

/// change the `n`th nibble to `nibble`
fn update_nibble(original_key: &HashValue, n: usize, nibble: u8) -> HashValue {
    assert!(nibble < 16);
    let mut key = original_key.to_vec();
    key[n / 2] = if n % 2 == 0 {
        key[n / 2] & 0x0f | nibble << 4
    } else {
        key[n / 2] & 0xf0 | nibble
    };
    HashValue::from_slice(&key).unwrap()
}
