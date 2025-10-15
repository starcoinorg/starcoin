// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use starcoin_crypto::HashValue;
use starcoin_dag::{
    blockdag::{BlockDAG, MineNewDagBlockInfo},
    consensusdb::prelude::{FlexiDagStorage, FlexiDagStorageConfig},
};
use starcoin_time_service::{MockTimeService, TimeService};
use starcoin_types::{block::BlockHeader, blockhash::KType, U256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

use crate::BlockEvent;

pub struct GhostAdpter {
    genesis: BlockHeader,
    dag: BlockDAG,
    pruning_point: HashValue,
    _storage_dir: TempDir,
    records: Vec<BlockRecord>,
    k: KType,
    virtual_tips: Vec<VirtualTip>,
    max_parents: usize,
}

impl GhostAdpter {
    pub fn new(k: KType, merge_depth: u64, max_parents_count: usize) -> Result<Self> {
        let time = MockTimeService::new();
        let genesis = BlockHeader::random()
            .as_builder()
            .with_difficulty(0.into())
            .with_timestamp(time.now_millis())
            .build();

        let db_tempdir = tempfile::tempdir()?;
        let config = FlexiDagStorageConfig::new();
        let dag_storage = FlexiDagStorage::create_from_path(db_tempdir.path(), config)?;
        let mut dag = BlockDAG::new(k, merge_depth, max_parents_count, dag_storage);
        dag.init_with_genesis(genesis.clone())?;
        let genesis_id = genesis.id();
        Ok(Self {
            genesis,
            dag,
            pruning_point: genesis_id,
            _storage_dir: db_tempdir,
            records: Vec::new(),
            k,
            virtual_tips: Vec::new(),
            max_parents: max_parents_count,
        })
    }

    pub fn genesis(&self) -> &BlockHeader {
        &self.genesis
    }

    pub fn genesis_id(&self) -> HashValue {
        self.genesis.id()
    }

    pub fn max_parents(&self) -> usize {
        self.max_parents
    }

    pub fn ghost_stats(&self, hash: HashValue) -> Result<Option<(u64, U256)>> {
        Ok(self
            .dag
            .ghostdata_by_hash(hash)?
            .map(|data| (data.blue_score, data.blue_work.clone())))
    }

    pub fn is_ancestor(&self, ancestor: HashValue, descendant: HashValue) -> Result<bool> {
        self.dag.check_ancestor_of(ancestor, descendant)
    }

    pub fn plan_next_block(&mut self) -> Result<MineNewDagBlockInfo> {
        self.dag
            .calc_mergeset_and_tips(self.pruning_point, self.genesis.id())
    }

    pub fn plan_with_parents(&mut self, parents: Vec<HashValue>) -> Result<MineNewDagBlockInfo> {
        let ghostdata = self.dag.ghostdata(&parents)?;
        Ok(MineNewDagBlockInfo {
            selected_parents: parents,
            ghostdata,
            pruning_point: self.pruning_point,
        })
    }

    pub fn commit_with_parents(
        &mut self,
        diff: U256,
        event: &BlockEvent,
        plan: MineNewDagBlockInfo,
        choice: ParentChoice,
    ) -> Result<HashValue> {
        let previous_pruning_point = self.pruning_point;
        // snapshot current tips before commit for renew_tips-like update
        let mut tips_snapshot = match self.dag.get_dag_state(previous_pruning_point) {
            Ok(state) => state.tips,
            Err(e) => {
                // In practice genesis bucket should exist; in case of a missing state, seed with genesis
                eprintln!(
                    "warn: failed to load dag state at {:?}: {:?}",
                    previous_pruning_point, e
                );
                vec![self.genesis.id()]
            }
        };
        let parents = match choice {
            ParentChoice::Honest => plan.selected_parents.clone(),
            ParentChoice::Subset(mut subset) => {
                subset.retain(|p| plan.selected_parents.contains(p));
                ensure!(!subset.is_empty(), "subset parent choice cannot be empty");
                subset
            }
            ParentChoice::Custom(custom) => custom,
        };

        let ghostdata = if parents == plan.selected_parents {
            Arc::new(plan.ghostdata)
        } else {
            Arc::new(self.dag.ghostdata(&parents)?)
        };
        let selected_parent = ghostdata.selected_parent;

        let block = BlockHeader::random()
            .as_builder()
            .with_parent_hash(selected_parent)
            .with_parents_hash(parents.clone())
            .with_difficulty(diff)
            .with_timestamp(event.header_time)
            .build();

        let block_id = block.id();
        self.dag.commit_trusted_block(block, ghostdata)?;
        // renew tips according to chain::renew_tips semantics
        // 1) remove old tips that are ancestors of the new block, then add the new block
        let mut new_tips = Vec::with_capacity(tips_snapshot.len() + 1);
        for tip in tips_snapshot.drain(..) {
            if !self.dag.check_ancestor_of(tip, block_id)? {
                new_tips.push(tip);
            }
        }
        new_tips.push(block_id);

        let virtual_info = self
            .dag
            .calc_mergeset_and_tips(previous_pruning_point, self.genesis.id())?;
        if virtual_info.pruning_point == previous_pruning_point {
            // Same bucket: save tips directly; DAG will merge/order/cap
            self.dag.save_dag_state(
                previous_pruning_point,
                starcoin_dag::consensusdb::consensus_state::DagState { tips: new_tips },
            )?;
        } else {
            // Bucket advanced: prune tips from previous bucket into the new pruning point, then save
            let pruned = self.dag.pruning_point_manager().prune(
                &starcoin_dag::consensusdb::consensus_state::DagState {
                    tips: new_tips.clone(),
                },
                previous_pruning_point,
                virtual_info.pruning_point,
            )?;
            self.dag.save_dag_state(
                virtual_info.pruning_point,
                starcoin_dag::consensusdb::consensus_state::DagState { tips: pruned },
            )?;
            self.pruning_point = virtual_info.pruning_point;
        }

        self.records.push(BlockRecord {
            block_id,
            parents,
            arrival_time: event.arrival_time,
            header_time: event.header_time,
            miner_id: event.miner_id,
        });
        let tip_id = virtual_info.ghostdata.selected_parent;
        let tip_data = Arc::new(virtual_info.ghostdata);
        self.virtual_tips.push(VirtualTip {
            tip: tip_id,
            blue_score: tip_data.blue_score,
            blue_work: tip_data.blue_work,
        });
        Ok(block_id)
    }

    pub fn records(&self) -> &[BlockRecord] {
        &self.records
    }

    pub fn audit_basic(&self, events: &[BlockEvent]) -> Result<()> {
        TopologyAudit::new(events, &self.records, self.genesis.id()).basic_checks()
    }

    pub fn audit_consensus(&self, events: &[BlockEvent]) -> Result<()> {
        self.audit_basic(events)?;

        let mut seen: HashMap<HashValue, (u64, U256)> = HashMap::new();
        let genesis_id = self.genesis.id();
        let genesis_data = self
            .dag
            .ghostdata_by_hash(genesis_id)?
            .ok_or_else(|| anyhow::anyhow!("missing ghost data for genesis"))?;
        seen.insert(
            genesis_id,
            (genesis_data.blue_score, genesis_data.blue_work),
        );

        for record in &self.records {
            let header = self.dag.storage.header_store.get_header(record.block_id)?;
            let difficulty = header.difficulty();
            ensure!(
                !difficulty.is_zero(),
                "block {:?} has zero difficulty",
                record.block_id
            );

            let ghostdata = self
                .dag
                .ghostdata_by_hash(record.block_id)?
                .ok_or_else(|| {
                    anyhow::anyhow!("missing ghost data for block {:?}", record.block_id)
                })?;

            ensure!(
                record
                    .parents
                    .iter()
                    .any(|p| *p == ghostdata.selected_parent),
                "block {:?} selected parent {:?} not in parent set {:?}",
                record.block_id,
                ghostdata.selected_parent,
                record.parents
            );

            for parent in &record.parents {
                ensure!(
                    seen.contains_key(parent),
                    "block {:?} references unknown parent {:?}",
                    record.block_id,
                    parent
                );
            }

            for blue in ghostdata.mergeset_blues.iter() {
                if *blue != record.block_id {
                    ensure!(
                        seen.contains_key(blue),
                        "block {:?} blue member {:?} unseen",
                        record.block_id,
                        blue
                    );
                }
            }

            for red in ghostdata.mergeset_reds.iter() {
                ensure!(
                    seen.contains_key(red),
                    "block {:?} red member {:?} unseen",
                    record.block_id,
                    red
                );
            }

            let (parent_score, parent_work) = seen
                .get(&ghostdata.selected_parent)
                .copied()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "selected parent {:?} not recorded",
                        ghostdata.selected_parent
                    )
                })?;

            ensure!(
                ghostdata.blue_score >= parent_score,
                "block {:?} blue score {} below parent {}",
                record.block_id,
                ghostdata.blue_score,
                parent_score
            );

            ensure!(
                ghostdata.blue_work >= parent_work,
                "block {:?} blue work {} below parent {}",
                record.block_id,
                ghostdata.blue_work,
                parent_work
            );

            for (hash, size) in ghostdata.blues_anticone_sizes.iter() {
                if *hash != record.block_id {
                    ensure!(
                        seen.contains_key(hash),
                        "block {:?} anticone member {:?} unseen",
                        record.block_id,
                        hash
                    );
                }
                ensure!(
                    *size <= self.k,
                    "block {:?} anticone size {} exceeds k {}",
                    record.block_id,
                    size,
                    self.k
                );
            }

            seen.insert(record.block_id, (ghostdata.blue_score, ghostdata.blue_work));
        }

        if let Some(last_virtual) = self.virtual_tips.last() {
            let dag_state = self.dag.get_dag_state(self.pruning_point)?;
            for tip in dag_state.tips {
                let Some(data) = self.dag.ghostdata_by_hash(tip)? else {
                    continue;
                };
                ensure!(
                    data.blue_score <= last_virtual.blue_score,
                    "tip {:?} blue score {} exceeds virtual {}",
                    tip,
                    data.blue_score,
                    last_virtual.blue_score
                );
                ensure!(
                    data.blue_work <= last_virtual.blue_work,
                    "tip {:?} blue work {} exceeds virtual {}",
                    tip,
                    data.blue_work,
                    last_virtual.blue_work
                );
            }
        }

        Ok(())
    }

    pub fn to_dot(&self) -> Result<String> {
        let mut buf =
            String::from("digraph G {\n  rankdir=LR;\n  node [shape=box, fontsize=10];\n");
        let mut selected_chain = HashSet::new();
        if let Some(last) = self.virtual_tips.last() {
            let mut cursor = last.tip;
            selected_chain.insert(cursor);
            while cursor != self.genesis.id() {
                if let Some(data) = self.dag.ghostdata_by_hash(cursor)? {
                    cursor = data.selected_parent;
                    if !selected_chain.insert(cursor) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        let mut label_cache: HashMap<HashValue, String> = HashMap::new();
        let mut node_cache: HashMap<HashValue, String> = HashMap::new();
        let mut label = |hash: &HashValue| {
            label_cache
                .entry(*hash)
                .or_insert_with(|| {
                    let hex = hash.to_hex();
                    format!("0x{}", &hex[..8])
                })
                .clone()
        };
        let mut node_id = |hash: &HashValue| {
            node_cache
                .entry(*hash)
                .or_insert_with(|| {
                    let hex = hash.to_hex();
                    format!("n{}", &hex[..8])
                })
                .clone()
        };

        let genesis_id = self.genesis.id();
        buf.push_str(&format!(
            "  {} [label=\"{}\\n(genesis)\", shape=oval, color=blue, penwidth=2];\n",
            node_id(&genesis_id),
            label(&genesis_id)
        ));

        for record in &self.records {
            let color = if selected_chain.contains(&record.block_id) {
                "color=green,penwidth=2"
            } else {
                "color=black"
            };
            buf.push_str(&format!(
                "  {} [label=\"{}\", {}];\n",
                node_id(&record.block_id),
                label(&record.block_id),
                color
            ));

            if let Some(ghostdata) = self.dag.ghostdata_by_hash(record.block_id)? {
                for parent in &record.parents {
                    let attrs = if ghostdata.mergeset_blues.contains(parent) {
                        "color=blue"
                    } else {
                        "color=red,style=dashed"
                    };
                    buf.push_str(&format!(
                        "  {} -> {} [{}];\n",
                        node_id(parent),
                        node_id(&record.block_id),
                        attrs
                    ));
                }
            }
        }

        buf.push_str("}\n");
        Ok(buf)
    }

    pub fn dump_dot(&self, name: &str) -> Result<PathBuf> {
        fs::create_dir_all("target/simnet-topology")?;
        let path = PathBuf::from(format!("target/simnet-topology/{}.dot", name));
        fs::write(&path, self.to_dot()?)?;
        Ok(path)
    }
}

#[derive(Clone, Debug)]
pub struct BlockRecord {
    pub block_id: HashValue,
    pub parents: Vec<HashValue>,
    pub arrival_time: u64,
    pub header_time: u64,
    pub miner_id: usize,
}

#[derive(Clone, Debug)]
pub struct VirtualTip {
    pub tip: HashValue,
    pub blue_score: u64,
    pub blue_work: U256,
}

#[derive(Clone, Debug)]
pub enum ParentChoice {
    Honest,
    Subset(Vec<HashValue>),
    Custom(Vec<HashValue>),
}

pub struct TopologyAudit<'a> {
    events: &'a [BlockEvent],
    records: &'a [BlockRecord],
    genesis: HashValue,
}

impl<'a> TopologyAudit<'a> {
    pub fn new(events: &'a [BlockEvent], records: &'a [BlockRecord], genesis: HashValue) -> Self {
        Self {
            events,
            records,
            genesis,
        }
    }

    pub fn basic_checks(&self) -> Result<()> {
        ensure!(
            self.events.len() == self.records.len(),
            "event count {} does not match committed blocks {}",
            self.events.len(),
            self.records.len()
        );

        let mut seen = HashSet::new();
        seen.insert(self.genesis);

        for (event, record) in self.events.iter().zip(self.records.iter()) {
            ensure!(
                record.arrival_time == event.arrival_time,
                "arrival time mismatch for miner {}: {} vs {}",
                record.miner_id,
                record.arrival_time,
                event.arrival_time
            );
            ensure!(
                record.header_time == event.header_time,
                "header time mismatch for miner {}",
                record.miner_id
            );
            ensure!(
                record.miner_id == event.miner_id,
                "miner mismatch: record {} vs event {}",
                record.miner_id,
                event.miner_id
            );

            for parent in &record.parents {
                ensure!(
                    seen.contains(parent),
                    "parent {:?} unknown when processing block {:?}",
                    parent,
                    record.block_id
                );
            }

            seen.insert(record.block_id);
        }

        Ok(())
    }
}

#[cfg(test)]
mod basic;
#[cfg(test)]
mod harness;
