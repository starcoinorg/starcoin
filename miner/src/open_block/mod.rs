use actix::prelude::*;
use actix::{Actor, Addr, Context};
use anyhow::{bail, format_err, Result};
use bus::{BusActor, Subscription};
use consensus::Consensus;
use crypto::hash::HashValue;
use logger::prelude::*;
use scs::SCSCodec;
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_vm_types::{chain_config::ChainNetwork, on_chain_config::EpochInfo};
use statedb::ChainStateDB;
use std::{collections::HashMap, convert::TryInto, sync::Arc};
use storage::Store;
use traits::ExcludedTxns;
use types::{
    account_address::AccountAddress,
    block::{BlockBody, BlockHeader, BlockInfo, BlockTemplate},
    block_metadata::BlockMetadata,
    error::BlockExecutorError,
    system_events::{NewBranch, NewHeadBlock},
    transaction::{
        SignedUserTransaction, Transaction, TransactionInfo, TransactionOutput, TransactionStatus,
    },
    vm_error::KeptVMStatus,
};

pub type UnclesActorAddress = Addr<UnclesActor>;

pub struct CreateBlockTemplateRequest {
    final_block_gas_limit: u64,
    author: AccountAddress,
    auth_key_prefix: Option<Vec<u8>>,
    user_txns: Vec<SignedUserTransaction>,
}

impl CreateBlockTemplateRequest {
    pub fn new(
        final_block_gas_limit: u64,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Self {
        Self {
            final_block_gas_limit,
            author,
            auth_key_prefix,
            user_txns,
        }
    }
}

impl
    Into<(
        u64,
        AccountAddress,
        Option<Vec<u8>>,
        Vec<SignedUserTransaction>,
    )> for CreateBlockTemplateRequest
{
    fn into(
        self,
    ) -> (
        u64,
        AccountAddress,
        Option<Vec<u8>>,
        Vec<SignedUserTransaction>,
    ) {
        (
            self.final_block_gas_limit,
            self.author,
            self.auth_key_prefix,
            self.user_txns,
        )
    }
}

pub struct CreateBlockTemplateResponse {
    block_template: BlockTemplate,
    txns: ExcludedTxns,
}

impl Into<(BlockTemplate, ExcludedTxns)> for CreateBlockTemplateResponse {
    fn into(self) -> (BlockTemplate, ExcludedTxns) {
        (self.block_template, self.txns)
    }
}

impl Message for CreateBlockTemplateRequest {
    type Result = Result<CreateBlockTemplateResponse>;
}

pub struct UnclesActor {
    bus: Addr<BusActor>,
    inner: Inner,
}

impl UnclesActor {
    pub fn launch(
        net: ChainNetwork,
        bus: Addr<BusActor>,
        storage: Arc<dyn Store>,
    ) -> UnclesActorAddress {
        UnclesActor::create(move |_ctx| {
            let inner = Inner::new(None, storage, net);
            UnclesActor { bus, inner }
        })
    }
}

impl Actor for UnclesActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<NewHeadBlock>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);

        let recipient = ctx.address().recipient::<NewBranch>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);

        info!("MinerActor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("MinerActor stopped");
    }
}

impl Handler<NewHeadBlock> for UnclesActor {
    type Result = ();

    fn handle(&mut self, msg: NewHeadBlock, _ctx: &mut Self::Context) -> Self::Result {
        self.inner.update_head(msg.0.header().id());
    }
}

impl Handler<NewBranch> for UnclesActor {
    type Result = ();

    fn handle(&mut self, msg: NewBranch, _ctx: &mut Self::Context) -> Self::Result {
        self.inner.insert_uncle((&*msg.0).clone());
    }
}

impl Handler<CreateBlockTemplateRequest> for UnclesActor {
    type Result = Result<CreateBlockTemplateResponse>;

    fn handle(
        &mut self,
        msg: CreateBlockTemplateRequest,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let (final_block_gas_limit, author, auth_key_prefix, user_txns) = msg.into();
        let (block_template, txns) = self.inner.create_block_template(
            final_block_gas_limit,
            author,
            auth_key_prefix,
            user_txns,
        )?;
        Ok(CreateBlockTemplateResponse {
            block_template,
            txns,
        })
    }
}

pub struct Inner {
    head_block_id: Option<HashValue>,
    parent_uncle: HashMap<HashValue, Vec<HashValue>>,
    uncles: HashMap<HashValue, BlockHeader>,
    _epoch_data: Option<EpochInfo>,
    storage: Arc<dyn Store>,
    net: ChainNetwork,
}

impl Inner {
    fn update_head(&mut self, head_block_id: HashValue) {
        self.head_block_id = Some(head_block_id);
    }

    fn insert_uncle(&mut self, uncle: BlockHeader) {
        if self.parent_uncle.contains_key(&uncle.parent_hash()) {
            self.parent_uncle
                .get_mut(&uncle.parent_hash())
                .unwrap()
                .push(uncle.id());
        } else {
            let mut empty = Vec::new();
            empty.push(uncle.id());
            self.parent_uncle.insert(uncle.parent_hash(), empty);
        }
        self.uncles.insert(uncle.id(), uncle);
    }

    fn new(
        previous_block_id: Option<HashValue>,
        storage: Arc<dyn Store>,
        net: ChainNetwork,
    ) -> Self {
        Inner {
            head_block_id: previous_block_id,
            parent_uncle: HashMap::new(),
            uncles: HashMap::new(),
            _epoch_data: None,
            storage,
            net,
        }
    }

    pub fn create_block_template(
        &self,
        final_block_gas_limit: u64,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        if let Some(previous_block_id) = self.head_block_id {
            if let Some(previous_header) =
                self.storage.get_block_header_by_hash(previous_block_id)?
            {
                //TODO: uncles
                let uncles = Vec::new();
                let mut opened_block = OpenedBlock::new(
                    self.storage.clone(),
                    previous_header,
                    final_block_gas_limit,
                    author,
                    auth_key_prefix,
                    self.net.consensus().now(),
                    uncles,
                )?;
                let excluded_txns = opened_block.push_txns(user_txns)?;
                let template = opened_block.finalize()?;
                return Ok((template, excluded_txns));
            }
        };

        Err(format_err!("create block template failed."))
    }
}

struct OpenedBlock {
    previous_block_info: BlockInfo,
    block_meta: BlockMetadata,
    gas_limit: u64,

    state: ChainStateDB,
    txn_accumulator: MerkleAccumulator,

    gas_used: u64,
    included_user_txns: Vec<SignedUserTransaction>,
    uncles: Vec<BlockHeader>,
}

impl OpenedBlock {
    pub fn new(
        storage: Arc<dyn Store>,
        previous_header: BlockHeader,
        block_gas_limit: u64,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        block_timestamp: u64,
        uncles: Vec<BlockHeader>,
    ) -> Result<Self> {
        let previous_block_id = previous_header.id();
        let block_info = storage
            .get_block_info(previous_block_id)?
            .ok_or_else(|| format_err!("Can not find block info by hash {}", previous_block_id))?;
        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let txn_accumulator = MerkleAccumulator::new(
            *txn_accumulator_info.get_accumulator_root(),
            txn_accumulator_info.get_frozen_subtree_roots().clone(),
            txn_accumulator_info.get_num_leaves(),
            txn_accumulator_info.get_num_nodes(),
            AccumulatorStoreType::Transaction,
            storage.clone().into_super_arc(),
        )?;
        // let block_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let chain_state =
            ChainStateDB::new(storage.into_super_arc(), Some(previous_header.state_root()));
        let block_meta = BlockMetadata::new(
            previous_block_id,
            block_timestamp,
            author,
            auth_key_prefix,
            uncles.len() as u64,
            previous_header.number + 1,
        );
        let mut opened_block = Self {
            previous_block_info: block_info,
            block_meta,
            gas_limit: block_gas_limit,

            state: chain_state,
            txn_accumulator,
            gas_used: 0,
            included_user_txns: vec![],
            uncles,
        };
        opened_block.initialize()?;
        Ok(opened_block)
    }

    /// Try to add `user_txns` into this block.
    /// Return any txns  not included, either txn is discarded, or block gas limit is reached.
    /// If error occurs during the processing, the `open_block` should be dropped,
    /// as the internal state may be corrupted.
    /// TODO: make the function can be called again even last call returns error.  
    pub fn push_txns(&mut self, user_txns: Vec<SignedUserTransaction>) -> Result<ExcludedTxns> {
        let mut txns: Vec<_> = user_txns
            .iter()
            .cloned()
            .map(Transaction::UserTransaction)
            .collect();

        let txn_outputs = {
            let gas_left = self
                .gas_limit
                .checked_sub(self.gas_used)
                .expect("block gas_used exceed block gas_limit");
            executor::execute_block_transactions(&self.state, txns.clone(), gas_left)?
        };

        let untouched_user_txns: Vec<SignedUserTransaction> = if txn_outputs.len() >= txns.len() {
            vec![]
        } else {
            txns.drain(txn_outputs.len()..)
                .map(|t| t.try_into().expect("user txn"))
                .collect()
        };

        let mut discard_txns: Vec<SignedUserTransaction> = Vec::new();
        debug_assert_eq!(txns.len(), txn_outputs.len());
        for (txn, output) in txns.into_iter().zip(txn_outputs.into_iter()) {
            let txn_hash = txn.id();
            match output.status() {
                TransactionStatus::Discard(status) => {
                    debug!("discard txn {}, vm status: {:?}", txn_hash, status);
                    discard_txns.push(txn.try_into().expect("user txn"));
                }
                TransactionStatus::Keep(status) => {
                    if status != &KeptVMStatus::Executed {
                        debug!("txn {:?} execute error: {:?}", txn_hash, status);
                    }
                    let gas_used = output.gas_used();
                    self.push_txn_and_state(txn_hash, output)?;
                    self.gas_used += gas_used;
                    self.included_user_txns
                        .push(txn.try_into().expect("user txn"));
                }
            };
        }
        Ok(ExcludedTxns {
            discarded_txns: discard_txns,
            untouched_txns: untouched_user_txns,
        })
    }

    /// Run blockmeta first
    fn initialize(&mut self) -> Result<()> {
        let block_metadata_txn = Transaction::BlockMetadata(self.block_meta.clone());
        let block_meta_txn_hash = block_metadata_txn.id();
        let mut results = executor::execute_transactions(&self.state, vec![block_metadata_txn])
            .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
        let output = results.pop().expect("execute txn has output");

        match output.status() {
            TransactionStatus::Discard(status) => {
                bail!("block_metadata txn is discarded, vm status: {:?}", status);
            }
            TransactionStatus::Keep(_) => {
                let _ = self.push_txn_and_state(block_meta_txn_hash, output)?;
            }
        };
        Ok(())
    }

    fn push_txn_and_state(
        &mut self,
        txn_hash: HashValue,
        output: TransactionOutput,
    ) -> Result<(HashValue, HashValue)> {
        let (write_set, events, gas_used, _, status) = output.into_inner();
        debug_assert!(matches!(status, TransactionStatus::Keep(_)));
        let status = status
            .status()
            .expect("TransactionStatus at here must been KeptVMStatus");
        self.state
            .apply_write_set(write_set)
            .map_err(BlockExecutorError::BlockChainStateErr)?;
        let txn_state_root = self
            .state
            .commit()
            .map_err(BlockExecutorError::BlockChainStateErr)?;

        let txn_info = TransactionInfo::new(
            txn_hash,
            txn_state_root,
            events.as_slice(),
            gas_used,
            status,
        );
        let (accumulator_root, _) = self.txn_accumulator.append(&[txn_info.id()])?;
        Ok((txn_state_root, accumulator_root))
    }

    /// Construct a block template for mining.
    pub fn finalize(self) -> Result<BlockTemplate> {
        let accumulator_root = self.txn_accumulator.root_hash();
        let state_root = self.state.state_root();
        let (parent_id, timestamp, author, auth_key_prefix, _uncles, number) =
            self.block_meta.into_inner();

        let (uncle_hash, uncles) = if !self.uncles.is_empty() {
            (
                Some(HashValue::sha3_256_of(&self.uncles.encode()?)),
                Some(self.uncles),
            )
        } else {
            (None, None)
        };
        let body = BlockBody::new(self.included_user_txns, uncles);
        let block_template = BlockTemplate::new(
            parent_id,
            self.previous_block_info
                .block_accumulator_info
                .accumulator_root,
            timestamp,
            number,
            author,
            auth_key_prefix,
            accumulator_root,
            state_root,
            self.gas_used,
            self.gas_limit,
            uncle_hash,
            body,
        );
        Ok(block_template)
    }
}
