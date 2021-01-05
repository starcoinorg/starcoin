// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    node_type::{LeafNode, Node, NodeKey},
    HashValueKey, NodeBatch, StaleNodeIndex, TreeReader, TreeUpdateBatch, TreeWriter,
};
use anyhow::{bail, ensure, Result};
use starcoin_crypto::HashValue;
use std::{
    collections::{hash_map::Entry, BTreeSet, HashMap},
    sync::RwLock,
};

#[derive(Default)]
pub struct MockTreeStore(
    RwLock<(
        HashMap<NodeKey, Node<HashValueKey>>,
        BTreeSet<StaleNodeIndex>,
    )>,
);

impl TreeReader<HashValueKey> for MockTreeStore {
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node<HashValueKey>>> {
        Ok(self.0.read().unwrap().0.get(node_key).cloned())
    }

    fn get_rightmost_leaf(&self) -> Result<Option<(NodeKey, LeafNode<HashValueKey>)>> {
        let locked = self.0.read().unwrap();
        let mut node_key_and_node: Option<(NodeKey, LeafNode<HashValueKey>)> = None;

        for (key, value) in locked.0.iter() {
            if let Node::Leaf(leaf_node) = value {
                if node_key_and_node.is_none()
                    || leaf_node.raw_key() > node_key_and_node.as_ref().unwrap().1.raw_key()
                {
                    node_key_and_node.replace((*key, leaf_node.clone()));
                }
            }
        }

        Ok(node_key_and_node)
    }
}

impl TreeWriter<HashValueKey> for MockTreeStore {
    fn write_node_batch(&self, node_batch: &NodeBatch<HashValueKey>) -> Result<()> {
        let mut locked = self.0.write().unwrap();
        for (node_key, node) in node_batch.clone() {
            assert_eq!(locked.0.insert(node_key, node), None);
        }
        Ok(())
    }
}

impl MockTreeStore {
    pub fn put_node(&self, node_key: NodeKey, node: Node<HashValueKey>) -> Result<()> {
        match self.0.write().unwrap().0.entry(node_key) {
            Entry::Occupied(o) => bail!("Key {:?} exists.", o.key()),
            Entry::Vacant(v) => {
                v.insert(node);
            }
        }
        Ok(())
    }

    fn put_stale_node_index(&self, index: StaleNodeIndex) -> Result<()> {
        let is_new_entry = self.0.write().unwrap().1.insert(index);
        ensure!(is_new_entry, "Duplicated retire log.");
        Ok(())
    }

    pub fn write_tree_update_batch(&self, batch: TreeUpdateBatch<HashValueKey>) -> Result<()> {
        batch
            .node_batch
            .into_iter()
            .map(|(k, v)| self.put_node(k, v))
            .collect::<Result<Vec<_>>>()?;
        batch
            .stale_node_index_batch
            .into_iter()
            .map(|i| self.put_stale_node_index(i))
            .collect::<Result<Vec<_>>>()?;
        Ok(())
    }

    pub fn purge_stale_nodes(&self, state_root_hash: HashValue) -> Result<()> {
        let mut wlocked = self.0.write().unwrap();

        // Only records retired before or at `least_readable_version` can be purged in order
        // to keep that version still readable.
        let to_prune = wlocked
            .1
            .iter()
            .take_while(|log| log.stale_since_version == state_root_hash)
            .cloned()
            .collect::<Vec<_>>();

        for log in to_prune {
            let removed = wlocked.0.remove(&log.node_key).is_some();
            ensure!(removed, "Stale node index refers to non-existent node.");
            wlocked.1.remove(&log);
        }

        Ok(())
    }

    pub fn num_nodes(&self) -> usize {
        self.0.read().unwrap().0.len()
    }
}
