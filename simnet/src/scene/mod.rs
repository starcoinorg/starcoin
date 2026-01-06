// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use starcoin_crypto::HashValue;
use starcoin_dag::{
    blockdag::{BlockDAG, MineNewDagBlockInfo},
    consensusdb::consensus_state::DagState,
    consensusdb::prelude::{FlexiDagStorage, FlexiDagStorageConfig},
    types::ghostdata::GhostdagData,
};
use starcoin_time_service::{MockTimeService, TimeService};
use starcoin_types::{block::BlockHeader, blockhash::KType, U256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
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
    profiler: Option<CommitProfiler>,
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
        let genesis_id = genesis.id();
        let mut dag = BlockDAG::new(k, merge_depth, max_parents_count, dag_storage, genesis_id);
        dag.init_with_genesis(genesis.clone())?;
        Ok(Self {
            genesis,
            dag,
            pruning_point: genesis_id,
            _storage_dir: db_tempdir,
            records: Vec::new(),
            k,
            virtual_tips: Vec::new(),
            max_parents: max_parents_count,
            profiler: if std::env::var("SIMNET_PROFILE").is_ok() {
                Some(CommitProfiler::default())
            } else {
                None
            },
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

    pub fn ghostdata(&self, hash: HashValue) -> Result<Option<Arc<GhostdagData>>> {
        self.dag.ghostdata_by_hash(hash)
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
        let mut sample = CommitSample::default();
        let total_start = Instant::now();

        let tips_start = Instant::now();
        let tips_snapshot = self.load_bucket_tips(previous_pruning_point);
        sample.load_tips = tips_start.elapsed();

        let select_start = Instant::now();
        let parents = Self::select_parents(&plan, choice)?;
        sample.select_parents = select_start.elapsed();

        let resolve_start = Instant::now();
        let ghostdata = self.resolve_ghostdata(&parents)?;
        sample.resolve = resolve_start.elapsed();
        let selected_parent = ghostdata.selected_parent;

        let build_start = Instant::now();
        let block = BlockHeader::random()
            .as_builder()
            .with_parent_hash(selected_parent)
            .with_parents_hash(parents.clone())
            .with_difficulty(diff)
            .with_timestamp(event.header_time)
            .build();
        sample.build_block = build_start.elapsed();

        let block_id = block.id();
        let commit_start = Instant::now();
        self.dag.commit_trusted_block(block, ghostdata.clone())?;
        sample.commit_block = commit_start.elapsed();

        let merge_start = Instant::now();
        let new_tips = self.merge_tips_with_block(tips_snapshot, block_id)?;
        sample.merge_tips = merge_start.elapsed();

        let virtual_start = Instant::now();
        let virtual_info = self
            .dag
            .calc_mergeset_and_tips(previous_pruning_point, self.genesis.id())?;
        sample.calc_virtual = virtual_start.elapsed();

        let persist_start = Instant::now();
        self.persist_tips(previous_pruning_point, virtual_info.pruning_point, new_tips)?;
        sample.persist_tips = persist_start.elapsed();

        let record_start = Instant::now();
        self.push_record(block_id, parents, event);
        self.push_virtual_tip(virtual_info);
        sample.record = record_start.elapsed();

        sample.total = total_start.elapsed();
        if let Some(profiler) = self.profiler.as_mut() {
            profiler.record(sample);
        }

        Ok(block_id)
    }

    fn load_bucket_tips(&self, bucket: HashValue) -> Vec<HashValue> {
        self.dag
            .get_dag_state(bucket)
            .map(|state| state.tips)
            .unwrap_or_else(|_| vec![self.genesis_id()])
    }

    fn select_parents(plan: &MineNewDagBlockInfo, choice: ParentChoice) -> Result<Vec<HashValue>> {
        Ok(match choice {
            ParentChoice::Honest => plan.selected_parents.clone(),
            ParentChoice::Subset(mut subset) => {
                subset.retain(|p| plan.selected_parents.contains(p));
                ensure!(!subset.is_empty(), "subset parent choice cannot be empty");
                subset
            }
            ParentChoice::Custom(custom) => custom,
        })
    }

    fn resolve_ghostdata(&mut self, parents: &[HashValue]) -> Result<Arc<GhostdagData>> {
        // Always recompute ghostdata: the harness plans blocks well before they are committed,
        // so the cached classification becomes outdated once newer blocks land.
        Ok(Arc::new(self.dag.ghostdata(parents)?))
    }

    fn merge_tips_with_block(
        &self,
        tips_snapshot: Vec<HashValue>,
        block_id: HashValue,
    ) -> Result<Vec<HashValue>> {
        let mut merged = Vec::with_capacity(tips_snapshot.len() + 1);
        for tip in tips_snapshot {
            if !self.dag.check_ancestor_of(tip, block_id)? {
                merged.push(tip);
            }
        }
        merged.push(block_id);
        Ok(merged)
    }

    fn persist_tips(
        &mut self,
        previous_pp: HashValue,
        next_pp: HashValue,
        tips: Vec<HashValue>,
    ) -> Result<()> {
        if next_pp == previous_pp {
            self.dag.save_dag_state(previous_pp, DagState { tips })?;
        } else {
            let state = DagState { tips };
            let pruned = self
                .dag
                .pruning_point_manager()
                .prune(&state, previous_pp, next_pp)?;
            self.dag
                .save_dag_state(next_pp, DagState { tips: pruned })?;
        }
        self.pruning_point = next_pp;
        Ok(())
    }

    fn push_record(&mut self, block_id: HashValue, parents: Vec<HashValue>, event: &BlockEvent) {
        self.records.push(BlockRecord {
            block_id,
            parents,
            arrival_time: event.arrival_time,
            header_time: event.header_time,
            miner_id: event.miner_id,
        });
    }

    fn push_virtual_tip(&mut self, info: MineNewDagBlockInfo) {
        let tip_data = info.ghostdata;
        self.virtual_tips.push(VirtualTip {
            tip: tip_data.selected_parent,
            blue_score: tip_data.blue_score,
            blue_work: tip_data.blue_work,
        });
    }

    pub fn records(&self) -> &[BlockRecord] {
        &self.records
    }

    pub fn virtual_tips(&self) -> &[VirtualTip] {
        &self.virtual_tips
    }

    pub fn header(&self, hash: HashValue) -> Result<BlockHeader> {
        Ok(self.dag.storage.header_store.get_header(hash)?)
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
        let mut blue_nodes: HashSet<HashValue> = HashSet::new();
        let mut red_nodes: HashSet<HashValue> = HashSet::new();
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
            if let Some(ghostdata) = self.dag.ghostdata_by_hash(record.block_id)? {
                blue_nodes.extend(ghostdata.mergeset_blues.iter().copied());
                red_nodes.extend(ghostdata.mergeset_reds.iter().copied());
            }
        }

        for record in &self.records {
            let node_attrs = if red_nodes.contains(&record.block_id) {
                "style=filled,fillcolor=\"#ffb7b7\",color=red"
            } else if selected_chain.contains(&record.block_id) {
                "style=filled,fillcolor=\"#174a96\",color=blue,penwidth=2,fontcolor=white"
            } else {
                "style=\"filled,dashed\",fillcolor=\"#b7d9ff\",color=blue"
            };

            buf.push_str(&format!(
                "  {} [label=\"{}\", {}];\n",
                node_id(&record.block_id),
                label(&record.block_id),
                node_attrs
            ));

            for parent in &record.parents {
                buf.push_str(&format!(
                    "  {} -> {} [color=blue,style=solid];\n",
                    node_id(parent),
                    node_id(&record.block_id)
                ));
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

impl Drop for GhostAdpter {
    fn drop(&mut self) {
        if let Some(profiler) = self.profiler.as_ref() {
            profiler.report();
        }
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

#[derive(Default, Clone, Debug)]
struct CommitSample {
    total: Duration,
    load_tips: Duration,
    select_parents: Duration,
    resolve: Duration,
    build_block: Duration,
    commit_block: Duration,
    merge_tips: Duration,
    calc_virtual: Duration,
    persist_tips: Duration,
    record: Duration,
}

#[derive(Default, Clone, Debug)]
struct CommitProfiler {
    total: Duration,
    load_tips: Duration,
    select_parents: Duration,
    resolve: Duration,
    build_block: Duration,
    commit_block: Duration,
    merge_tips: Duration,
    calc_virtual: Duration,
    persist_tips: Duration,
    record: Duration,
    count: usize,
}

impl CommitProfiler {
    fn record(&mut self, sample: CommitSample) {
        self.total += sample.total;
        self.load_tips += sample.load_tips;
        self.select_parents += sample.select_parents;
        self.resolve += sample.resolve;
        self.build_block += sample.build_block;
        self.commit_block += sample.commit_block;
        self.merge_tips += sample.merge_tips;
        self.calc_virtual += sample.calc_virtual;
        self.persist_tips += sample.persist_tips;
        self.record += sample.record;
        self.count += 1;
    }

    fn report(&self) {
        if self.count == 0 || std::env::var("SIMNET_PROFILE").is_err() {
            return;
        }
        let ms = |d: Duration| d.as_secs_f64() * 1_000.0;
        println!(
            "[simnet::commit] blocks={} total={:.2}ms avg={:.2}ms load_tips={:.2}ms select={:.2}ms resolve={:.2}ms build={:.2}ms commit={:.2}ms merge={:.2}ms calc_virtual={:.2}ms persist={:.2}ms record={:.2}ms",
            self.count,
            ms(self.total),
            ms(self.total) / self.count as f64,
            ms(self.load_tips),
            ms(self.select_parents),
            ms(self.resolve),
            ms(self.build_block),
            ms(self.commit_block),
            ms(self.merge_tips),
            ms(self.calc_virtual),
            ms(self.persist_tips),
            ms(self.record)
        );
    }
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
mod common_prefix;
#[cfg(test)]
mod harness;
