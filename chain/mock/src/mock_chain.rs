// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use starcoin_account_api::AccountInfo;
use starcoin_chain::{BlockChain, ChainReader, ChainWriter};
use starcoin_config::ChainNetwork;
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::{BlockDAG, MineNewDagBlockInfo};
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{Storage, Store};
use starcoin_storage::{Storage2, Store2};
use starcoin_types::block::{Block, BlockHeader, ExecutedBlock};
use starcoin_types::blockhash::KType;
use starcoin_types::startup_info::ChainInfo;
use std::sync::Arc;

pub struct MockChain {
    net: ChainNetwork,
    head: BlockChain,
    miner: AccountInfo,
}

impl MockChain {
    pub fn new(net: ChainNetwork) -> Result<Self> {
        let (storage, storage2, chain_info, _, dag) =
            Genesis::init_storage_for_test(&net).expect("init storage by genesis fail.");

        let chain = BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage,
            storage2,
            None,
            dag,
        )?;
        let miner = AccountInfo::random();
        Ok(Self::new_inner(net, chain, miner))
    }

    pub fn new_and_get_storage2(net: ChainNetwork) -> Result<(Self, Arc<Storage2>)> {
        let (storage, storage2, chain_info, _, dag) =
            Genesis::init_storage_for_test(&net).expect("init storage by genesis fail.");

        let storage2_clone = storage2.clone();
        let chain = BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage,
            storage2,
            None,
            dag,
        )?;
        let miner = AccountInfo::random();
        Ok((Self::new_inner(net, chain, miner), storage2_clone))
    }

    pub fn new_with_storage(
        net: ChainNetwork,
        storage: Arc<Storage>,
        storage2: Arc<Storage2>,
        head_block_hash: HashValue,
        miner: AccountInfo,
        dag: BlockDAG,
    ) -> Result<Self> {
        let chain = BlockChain::new(
            net.time_service(),
            head_block_hash,
            storage,
            storage2,
            None,
            dag,
        )?;
        Ok(Self::new_inner(net, chain, miner))
    }

    pub fn new_with_chain(net: ChainNetwork, chain: BlockChain) -> Result<Self> {
        let miner = AccountInfo::random();
        Ok(Self::new_inner(net, chain, miner))
    }

    pub fn new_with_genesis_for_test(
        net: ChainNetwork,
        genesis: Genesis,
        k: KType,
    ) -> Result<Self> {
        let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance())?);
        let storage2 = Arc::new(Storage2(storage.clone()));
        let genesis_hash = genesis.block().id();
        let dag = BlockDAG::create_for_testing_with_parameters(k, genesis_hash)?;
        let chain_info =
            genesis.execute_genesis_block(&net, storage.clone(), storage2.clone(), dag.clone())?;

        let chain = BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage.clone(),
            storage2,
            None,
            dag,
        )?;

        let miner = AccountInfo::random();
        Ok(Self::new_inner(net, chain, miner))
    }

    fn new_inner(net: ChainNetwork, head: BlockChain, miner: AccountInfo) -> Self {
        Self { net, head, miner }
    }

    pub fn net(&self) -> &ChainNetwork {
        &self.net
    }

    pub fn head(&self) -> &BlockChain {
        &self.head
    }

    pub fn chain_info(&self) -> ChainInfo {
        self.head.info()
    }

    pub fn get_storage(&self) -> Arc<dyn Store> {
        self.head.get_storage()
    }

    pub fn get_storage2(&self) -> Arc<dyn Store2> {
        self.head.get_storage2()
    }

    pub fn fork_new_branch(&self, head_id: Option<HashValue>) -> Result<BlockChain> {
        let block_id = match head_id {
            Some(id) => id,
            None => self.head.current_header().id(),
        };
        assert!(self.head.has_dag_block(block_id)?);
        BlockChain::new(
            self.head.time_service(),
            block_id,
            self.head.get_storage(),
            self.head.get_storage2(),
            None,
            self.head.dag(),
        )
    }

    pub fn fork(&self, head_id: Option<HashValue>) -> Result<MockChain> {
        let chain = self.fork_new_branch(head_id)?;
        Ok(Self {
            head: chain,
            net: self.net.clone(),
            miner: AccountInfo::random(),
        })
    }

    pub fn select_head(&mut self, new_block: Block) -> Result<()> {
        //TODO reuse WriteChainService's select_head logic.
        // new block should be execute and save to storage.
        let new_block_id = new_block.id();
        let branch = BlockChain::new(
            self.net.time_service(),
            new_block_id,
            self.head.get_storage(),
            self.head.get_storage2(),
            None,
            self.head.dag(),
        )?;
        let branch_total_difficulty = branch.get_total_difficulty()?;
        let head_total_difficulty = self.head.get_total_difficulty()?;
        if branch_total_difficulty > head_total_difficulty {
            self.head = branch;
            debug!("Change to new head: {:?}", self.head.current_header());
            self.net
                .time_service()
                .adjust(new_block.header().timestamp());
        } else {
            debug!(
                "New block({:?})'s total_difficulty({:?}) <= head's total_difficulty({:?})",
                new_block_id, branch_total_difficulty, head_total_difficulty
            );
        }
        Ok(())
    }

    pub fn produce(&self) -> Result<Block> {
        let (template, _) = self
            .head
            .create_block_template_simple(*self.miner.address())?;
        self.head
            .consensus()
            .create_block(template, self.net.time_service().as_ref())
    }

    pub fn apply(&mut self, block: Block) -> Result<()> {
        if self.head.current_header().id() != block.header().parent_hash() {
            self.head = self.head.fork(block.parent_hash())?;
        }
        self.head.apply(block)?;
        Ok(())
    }

    pub fn produce_and_apply(&mut self) -> Result<BlockHeader> {
        let block = self.produce()?;
        let header = block.header().clone();
        self.apply(block)?;
        Ok(header)
    }

    pub fn produce_and_apply_times(&mut self, times: u64) -> Result<()> {
        for _i in 0..times {
            self.produce_and_apply()?;
        }
        Ok(())
    }

    pub fn produce_and_apply_times_for_fork(
        &mut self,
        fork_point: BlockHeader,
        times: u64,
    ) -> Result<BlockHeader> {
        let mut parent_header = fork_point;
        let mut tips = vec![parent_header.id()];
        let mut last = parent_header.clone();
        for _i in 0..times {
            let block_header = self.produce_and_apply_by_tips(parent_header, tips)?;
            parent_header = block_header.clone();
            tips = vec![block_header.id()];
            last = block_header.clone();
        }
        Ok(last)
    }

    pub fn connect(&mut self, executed_block: ExecutedBlock) -> Result<()> {
        self.head.connect(executed_block)?;
        Ok(())
    }

    pub fn produce_and_apply_with_tips_for_times(
        &mut self,
        times: u64,
    ) -> Result<Vec<ExecutedBlock>> {
        let mut blocks = Vec::new();
        for _i in 0..times {
            let header = self.produce_and_apply()?;
            let block = self
                .head
                .get_storage()
                .get_block_by_hash(header.id())?
                .unwrap();
            let block_info = self
                .head
                .get_storage()
                .get_block_info(header.id())?
                .unwrap();
            // Get the actual MultiState from storage using the standard method
            let multi_state = self.head.get_storage().get_vm_multi_state(header.id())?;
            let executed_block = ExecutedBlock::new(block, block_info, multi_state);
            blocks.push(executed_block);
        }
        Ok(blocks)
    }

    pub fn produce_and_apply_by_tips(
        &mut self,
        parent_header: BlockHeader,
        tips: Vec<HashValue>,
    ) -> Result<BlockHeader> {
        let block = self.produce_block_by_tips(parent_header, tips)?;
        let header = block.header().clone();
        self.apply(block)?;
        Ok(header)
    }

    pub fn produce_block_by_tips(
        &mut self,
        parent_header: BlockHeader,
        tips: Vec<HashValue>,
    ) -> Result<Block> {
        if self.head().current_header().id() != parent_header.id() {
            self.head = self.head.fork(parent_header.id())?;
        }

        // TODO: API Design Issue - parent_header parameter is redundant after fork()
        //
        // The current API design has a fundamental issue:
        // 1. We call fork(parent_header.id()) to switch to the historical view
        // 2. After fork(), the chain head IS the parent we want to build on
        // 3. Passing parent_header again to create_block_template is redundant
        // 4. dual-verse-dag's strict validation exposes this redundancy
        //
        // Better API would be:
        //   chain.fork(parent_id)?;
        //   chain.create_block_template(author, txns, uncles, gas_limit, tips, pruning_point)?;
        //   // parent_header derived automatically from current head after fork
        //
        // For now, we ensure consistency by using ghostdata.selected_parent
        let tips_ghostdata = self.head.dag().ghost_dag_manager().ghostdag(&tips)?;
        let consistent_parent_header = self
            .head()
            .get_storage()
            .get_block_header_by_hash(tips_ghostdata.selected_parent)?
            .ok_or_else(|| {
                format_err!(
                    "Cannot find block header by hash: {:?}",
                    tips_ghostdata.selected_parent
                )
            })?;

        let (block_template, _) = self.head.create_block_template(
            *self.miner.address(),
            Some(consistent_parent_header), // Use consistent parent from tips
            Vec::new(),                     // user_txns
            None,                           // uncles
            None,                           // block_gas_limit
            Some(tips),                     // tips
            HashValue::zero(),              // pruning_point
        )?;
        let new_block = self
            .head
            .consensus()
            .create_block(block_template, self.net.time_service().as_ref())?;
        Ok(new_block)
    }

    pub fn produce_fork_chain(&mut self, one_count: u64, two_count: u64) -> Result<()> {
        let start_header = self.head.current_header();

        let mut parent_one = start_header.clone();
        for _i in 0..one_count {
            let new_block =
                self.produce_block_by_tips(parent_one.clone(), vec![parent_one.id()])?;
            parent_one = new_block.header().clone();
            self.apply(new_block)?;
        }

        let mut parent_two = start_header;
        for _i in 0..two_count {
            let new_block =
                self.produce_block_by_tips(parent_two.clone(), vec![parent_two.id()])?;
            parent_two = new_block.header().clone();
            self.apply(new_block)?;
        }

        // Create a meetup block that has both branches as parents
        let meetup_block = if one_count < two_count {
            self.produce_block_by_tips(parent_two.clone(), vec![parent_one.id(), parent_two.id()])?
        } else {
            self.produce_block_by_tips(parent_one.clone(), vec![parent_one.id(), parent_two.id()])?
        };
        let new_header_id = meetup_block.header().id();
        self.apply(meetup_block)?;

        assert_eq!(self.head.current_header().id(), new_header_id);

        Ok(())
    }

    pub fn produce_block_for_pruning(&mut self) -> Result<Block> {
        let tips = self.head.get_dag_state()?.tips;
        let ghostdata = self.head.dag().ghost_dag_manager().ghostdag(&tips)?;
        let selected_header = self
            .head()
            .get_storage()
            .get_block_header_by_hash(ghostdata.selected_parent)?
            .ok_or_else(|| {
                format_err!(
                    "Cannot find block header by hash: {:?}",
                    ghostdata.selected_parent
                )
            })?;

        let previous_pruning = if selected_header.pruning_point() == HashValue::zero() {
            self.head().get_storage().get_genesis()?.unwrap()
        } else {
            selected_header.pruning_point()
        };

        let MineNewDagBlockInfo {
            selected_parents: pruned_tips,
            ghostdata: pruned_ghostdata,
            pruning_point,
        } = self
            .head
            .dag()
            .calc_mergeset_and_tips(previous_pruning, self.head().get_genesis_hash())?;

        // Calculate the selected parent from pruned_tips to ensure consistency
        let pruned_tips_ghostdata = self.head.dag().ghost_dag_manager().ghostdag(&pruned_tips)?;
        let pruned_selected_parent_header = self
            .head()
            .get_storage()
            .get_block_header_by_hash(pruned_tips_ghostdata.selected_parent)?
            .ok_or_else(|| {
                format_err!(
                    "Cannot find selected parent header by hash: {:?}",
                    pruned_tips_ghostdata.selected_parent
                )
            })?;

        let (template, _) = self.head.create_block_template(
            *self.miner.address(),
            Some(pruned_selected_parent_header), // Use selected parent from pruned_tips for consistency
            vec![],
            Some(
                pruned_ghostdata
                    .mergeset_blues
                    .get(1..)
                    .unwrap_or(&[])
                    .iter()
                    .map(|block_id| {
                        self.head()
                            .get_storage()
                            .get_block_header_by_hash(*block_id)?
                            .ok_or_else(|| {
                                format_err!("Block header not found for hash: {:?}", block_id)
                            })
                    })
                    .collect::<Result<Vec<_>>>()?,
            ), // Use uncles from pruned ghostdata
            None,
            Some(pruned_tips), // Use calculated tips to control parents
            pruning_point,
        )?;
        self.head
            .consensus()
            .create_block(template, self.net.time_service().as_ref())
    }

    pub fn produce_block_by_params(
        &mut self,
        parent_header: BlockHeader,
        tips: Vec<HashValue>,
        pruning_point: HashValue,
    ) -> Result<Block> {
        let (block_template, _) = self.head.create_block_template(
            *self.miner.address(),
            Some(parent_header),
            Vec::new(),
            None,
            None,
            Some(tips),
            pruning_point,
        )?;

        let new_block = self
            .head
            .consensus()
            .create_block(block_template, self.net.time_service().as_ref())?;
        Ok(new_block)
    }

    pub fn miner(&self) -> &AccountInfo {
        &self.miner
    }
}
