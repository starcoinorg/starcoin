// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::remote_state::RemoteRpcAsyncClient;
use anyhow::{anyhow, bail, Result};
use dashmap::DashMap;
use futures::executor::block_on;
use jsonrpc_core::futures_util::{FutureExt, TryFutureExt};
use log::debug;
use starcoin_abi_decoder::decode_txn_payload;
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
use starcoin_config::{BuiltinNetworkID, ChainNetworkID};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::{ChainApi, GetBlockOption};
use starcoin_rpc_api::chain::{ChainApiClient, GetBlocksOption};
use starcoin_rpc_api::types::{
    BlockInfoView, BlockTransactionsView, BlockView, ChainId, ChainInfoView,
    SignedUserTransactionView, TransactionInfoView, TransactionView,
};
use starcoin_rpc_api::FutureResult;
use starcoin_rpc_server::module::map_err;
use starcoin_state_api::StateView;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{
    BlockStore, BlockTransactionInfoStore, ContractEventStore, Storage, Store, TransactionStore,
};
use starcoin_types::block::{Block, BlockInfo, BlockNumber};
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::transaction::{Transaction, TransactionInfo, TransactionOutput};
use starcoin_vm_types::access_path::AccessPath;
use std::hash::Hash;
use std::option::Option::{None, Some};
use std::sync::{Arc, Mutex};

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct ChainStatusWithBlock {
    pub status: ChainStatus,
    pub head: Block,
}

// #[derive(Clone)]
pub struct ForkBlockChain {
    remote_client: Option<Arc<RemoteRpcAsyncClient>>,
    storage: Arc<Storage>,
    fork_number: u64,
    current_number: u64,
    status: Option<ChainStatusWithBlock>,
    number_hash_map: DashMap<u64, HashValue>,
    txn_accumulator: MerkleAccumulator,
    state_root: Arc<Mutex<HashValue>>,
    head_block_hash: HashValue,
}

impl ForkBlockChain {
    pub fn new(state_root: Arc<Mutex<HashValue>>) -> Result<Self> {
        Self::new_inner(0, None, state_root, HashValue::zero())
    }

    pub fn fork(
        remote_client: Arc<RemoteRpcAsyncClient>,
        fork_number: u64,
        fork_block_hash: HashValue,
        state_root: Arc<Mutex<HashValue>>,
    ) -> Result<Self> {
        Self::new_inner(
            fork_number,
            Some(remote_client),
            state_root,
            fork_block_hash,
        )
    }
    // Mock chain fork from remote_client if fork_number > 0
    fn new_inner(
        fork_number: u64,
        remote_client: Option<Arc<RemoteRpcAsyncClient>>,
        state_root: Arc<Mutex<HashValue>>,
        head_block_hash: HashValue,
    ) -> Result<Self> {
        let storage_instance = StorageInstance::new_cache_instance();
        let storage = Arc::new(Storage::new(storage_instance)?);

        let accumulator_store = storage.get_accumulator_store(AccumulatorStoreType::Transaction);
        let txn_accumulator = match remote_client.clone() {
            Some(client) => {
                let block_info: Option<BlockInfo> = block_on(
                    client
                        .get_chain_client()
                        .get_block_info_by_number(fork_number),
                )
                .map_err(|e| anyhow!("{}", e))?
                .map(|view| view.into_info());
                match block_info {
                    Some(block) => MerkleAccumulator::new_with_info(
                        block.txn_accumulator_info,
                        accumulator_store,
                    ),
                    None => MerkleAccumulator::new_empty(accumulator_store),
                }
            }
            None => MerkleAccumulator::new_empty(accumulator_store),
        };
        Ok(Self {
            remote_client,
            storage,
            fork_number,
            current_number: fork_number,
            status: None,
            number_hash_map: DashMap::new(),
            txn_accumulator,
            state_root,
            head_block_hash,
        })
    }

    fn merge_tips(&self, block: &Block) -> Option<Vec<HashValue>> {
        let tips = self
            .status
            .as_ref()
            .and_then(|status| status.status.tips_hash().clone());
        let parents = block.header.parents_hash();
        match (tips, parents) {
            (Some(mut tips), Some(parents)) => {
                for hash in parents {
                    tips.retain(|x| *x != hash);
                }
                tips.push(block.id());
                Some(tips)
            }
            _ => None,
        }
    }

    pub fn add_new_block(&mut self, mut block: Block) -> Result<()> {
        block.header = block.header().as_builder().build();

        let block_accumulator = MerkleAccumulator::new_empty(
            self.storage
                .get_accumulator_store(AccumulatorStoreType::Block),
        );
        let block_info = BlockInfo::new(
            block.header.id(),
            block.header.difficulty(),
            self.txn_accumulator.get_info(),
            block_accumulator.get_info(),
        );
        self.current_number = block.header().number();
        self.head_block_hash = block.header().id();
        self.number_hash_map
            .insert(self.current_number, block.header().id());
        self.status = Some(ChainStatusWithBlock {
            status: ChainStatus::new(
                block.header().clone(),
                block_info.clone(),
                self.merge_tips(&block),
            ),
            head: block.clone(),
        });
        self.storage.save_block_info(block_info)?;
        self.storage.commit_block(block)?;
        Ok(())
    }

    pub fn add_new_txn(&mut self, txn: Transaction, output: TransactionOutput) -> Result<()> {
        let txn_hash = txn.id();
        let state_root = *self.state_root.lock().unwrap();
        let (_, _, events, gas_used, status) = output.into_inner();
        let status = status
            .status()
            .expect("TransactionStatus at here must been KeptVMStatus");
        let txn_info =
            TransactionInfo::new(txn_hash, state_root, events.as_slice(), gas_used, status);
        self.txn_accumulator.append(&[txn_info.id()])?;

        self.storage.save_contract_events(txn_hash, events)?;
        self.storage.save_transaction(txn)?;
        Ok(())
    }

    pub fn txn_accumulator_root(&self) -> HashValue {
        self.txn_accumulator.root_hash()
    }

    pub fn head_block_hash(&self) -> HashValue {
        self.head_block_hash
    }

    fn remote_chain_client(&self) -> Option<ChainApiClient> {
        self.remote_client
            .clone()
            .map(|client| client.get_chain_client().clone())
    }
}

pub struct MockChainApi {
    pub chain: Arc<Mutex<ForkBlockChain>>,
}

impl MockChainApi {
    pub fn new(chain: Arc<Mutex<ForkBlockChain>>) -> Self {
        Self { chain }
    }
}

impl ChainApi for MockChainApi {
    fn id(&self) -> jsonrpc_core::Result<ChainId> {
        Ok(ChainId::from(&ChainNetworkID::Builtin(
            BuiltinNetworkID::Dev,
        )))
    }

    fn info(&self) -> FutureResult<ChainInfoView> {
        let chain = self.chain.lock().unwrap();
        let status = chain.status.clone();
        let client = chain.remote_chain_client();
        let fut = async move {
            match status {
                Some(status) => Ok(ChainInfoView::from(ChainInfo::new(
                    status.head.header().chain_id(),
                    HashValue::random(),
                    status.status,
                ))),
                None => match client {
                    Some(client) => client.info().await.map_err(|e| anyhow!("{}", e)),
                    None => bail!("No block generated."),
                },
            }
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_block_by_hash(
        &self,
        hash: HashValue,
        option: Option<GetBlockOption>,
    ) -> FutureResult<Option<BlockView>> {
        let chain = self.chain.lock().unwrap();
        let storage = chain.storage.clone();
        let client = chain.remote_chain_client();
        let decode = option.unwrap_or_default().decode;
        let raw = option.unwrap_or_default().raw;
        let status = chain.status.clone();
        let fut = async move {
            match storage.get_block_by_hash(hash)? {
                Some(block) => {
                    let mut block_view = BlockView::try_from_block(block, false, raw)?;
                    if decode {
                        debug_assert!(status.is_some());
                        let state = ChainStateDB::new(
                            storage,
                            Some(status.unwrap().status.head().state_root()),
                        );
                        try_decode_block_txns(&state, &mut block_view)?;
                    }
                    Ok(Some(block_view))
                }
                None => match client {
                    Some(client) => client
                        .get_block_by_hash(hash, option)
                        .await
                        .map_err(|e| anyhow!("{}", e)),
                    None => Ok(None),
                },
            }
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_block_by_number(
        &self,
        number: BlockNumber,
        option: Option<GetBlockOption>,
    ) -> FutureResult<Option<BlockView>> {
        let chain = self.chain.lock().unwrap();
        let client = chain.remote_chain_client();
        let fork_number = chain.fork_number;
        let current_number = chain.current_number;
        let number_hash_map = chain.number_hash_map.clone();
        let storage = chain.storage.clone();
        let decode = option.unwrap_or_default().decode;
        let raw = option.unwrap_or_default().raw;
        let status = chain.status.clone();
        let fut = async move {
            if number <= fork_number {
                debug_assert!(client.is_some());
                client
                    .unwrap()
                    .get_block_by_number(number, option)
                    .await
                    .map_err(|e| anyhow!("{}", e))
            } else if number <= current_number {
                let hash = number_hash_map.get(&number).map(|h| *h);
                let block_view = match hash {
                    Some(hash) => match storage.get_block_by_hash(hash)? {
                        Some(block) => {
                            let mut block_view = BlockView::try_from_block(block, false, raw)?;
                            if decode {
                                debug_assert!(status.is_some());
                                let state = ChainStateDB::new(
                                    storage,
                                    Some(status.unwrap().status.head().state_root()),
                                );
                                try_decode_block_txns(&state, &mut block_view)?;
                            }
                            Some(block_view)
                        }
                        None => None,
                    },
                    None => None,
                };
                Ok(block_view)
            } else {
                Ok(None)
            }
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_blocks_by_number(
        &self,
        _number: Option<BlockNumber>,
        _count: u64,
        _option: Option<GetBlocksOption>,
    ) -> FutureResult<Vec<BlockView>> {
        let fut = async move {
            bail!("not implemented.");
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_block_info_by_number(&self, number: BlockNumber) -> FutureResult<Option<BlockInfoView>> {
        let chain = self.chain.lock().unwrap();
        let client = chain.remote_chain_client();
        let fork_number = chain.fork_number;
        let current_number = chain.current_number;
        let number_hash_map = chain.number_hash_map.clone();
        let storage = chain.storage.clone();
        let fut = async move {
            if number <= fork_number {
                debug_assert!(client.is_some());
                client
                    .unwrap()
                    .get_block_info_by_number(number)
                    .await
                    .map_err(|e| anyhow!("{}", e))
            } else if number <= current_number {
                let hash = number_hash_map.get(&number).map(|h| *h);
                let block_view = match hash {
                    Some(hash) => match storage.get_block_info(hash)? {
                        Some(b) => Some(BlockInfoView::try_from(b)?),
                        None => None,
                    },
                    None => None,
                };
                Ok(block_view)
            } else {
                Ok(None)
            }
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_transaction(
        &self,
        transaction_hash: HashValue,
        option: Option<starcoin_rpc_api::chain::GetTransactionOption>,
    ) -> starcoin_rpc_api::FutureResult<Option<TransactionView>> {
        let chain = self.chain.lock().unwrap();
        let storage = chain.storage.clone();
        let client = chain.remote_chain_client();
        let status = chain.status.clone();
        let decode_payload = option.unwrap_or_default().decode;
        let fut = async move {
            match storage.get_transaction(transaction_hash)? {
                Some(txn) => {
                    // WATNING: the txn here is not in any blocks, use head block instead.
                    // TODO: How to handle the txns not in any blocks.
                    let block = status.clone().unwrap().head;

                    let mut txn = TransactionView::new(txn, &block)?;
                    if decode_payload {
                        let state = ChainStateDB::new(
                            storage,
                            Some(status.unwrap().status.head().state_root()),
                        );
                        if let Some(txn) = txn.user_transaction.as_mut() {
                            try_decode_txn_payload(&state, txn)?;
                        }
                    }
                    Ok(Some(txn))
                }
                None => match client {
                    Some(client) => client
                        .get_transaction(transaction_hash, option)
                        .await
                        .map_err(|e| anyhow!("{}", e)),
                    None => Ok(None),
                },
            }
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_transaction_info(
        &self,
        transaction_hash: HashValue,
    ) -> starcoin_rpc_api::FutureResult<Option<TransactionInfoView>> {
        let chain = self.chain.lock().unwrap();
        let storage = chain.storage.clone();
        let client = chain.remote_chain_client();
        let fut = async move {
            match storage.get_transaction_info(transaction_hash)? {
                Some(_txn) => {
                    debug_assert!(false); // Never saved transaction info.
                    unimplemented!()
                }
                None => match client {
                    Some(client) => client
                        .get_transaction_info(transaction_hash)
                        .await
                        .map_err(|e| anyhow!("{}", e)),
                    None => Ok(None),
                },
            }
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_block_txn_infos(
        &self,
        _block_hash: HashValue,
    ) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::TransactionInfoView>> {
        let fut = async move {
            bail!("not implemented.");
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_txn_info_by_block_and_index(
        &self,
        _block_hash: HashValue,
        _idx: u64,
    ) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::TransactionInfoView>> {
        let fut = async move {
            bail!("not implemented.");
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_events_by_txn_hash(
        &self,
        _txn_hash: HashValue,
        _option: Option<starcoin_rpc_api::chain::GetEventOption>,
    ) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::TransactionEventResponse>>
    {
        let fut = async move {
            bail!("not implemented.");
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_events(
        &self,
        _filter: starcoin_rpc_api::types::pubsub::EventFilter,
        _option: Option<starcoin_rpc_api::chain::GetEventOption>,
    ) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::TransactionEventResponse>>
    {
        let fut = async move {
            bail!("not implemented.");
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_headers(
        &self,
        _ids: Vec<HashValue>,
    ) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::BlockHeaderView>> {
        let fut = async move {
            bail!("not implemented.");
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_transaction_infos(
        &self,
        _start_global_index: u64,
        _reverse: bool,
        _max_size: u64,
    ) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::TransactionInfoView>> {
        let fut = async move {
            bail!("not implemented.");
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_transaction_proof(
        &self,
        _block_hash: HashValue,
        _transaction_global_index: u64,
        _event_index: Option<u64>,
        _access_path: Option<starcoin_rpc_api::types::StrView<AccessPath>>,
    ) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::TransactionInfoWithProofView>>
    {
        let fut = async move {
            bail!("not implemented.");
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_transaction_proof_raw(
        &self,
        _block_hash: HashValue,
        _transaction_global_index: u64,
        _event_index: Option<u64>,
        _access_path: Option<starcoin_rpc_api::types::StrView<AccessPath>>,
    ) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::StrView<Vec<u8>>>> {
        let fut = async move {
            bail!("not implemented.");
        };
        Box::pin(fut.boxed().map_err(map_err))
    }
}

fn try_decode_block_txns(state: &dyn StateView, block: &mut BlockView) -> anyhow::Result<()> {
    if let BlockTransactionsView::Full(txns) = &mut block.body {
        for txn in txns.iter_mut() {
            try_decode_txn_payload(state, txn)?;
        }
    }
    Ok(())
}

fn try_decode_txn_payload(
    state: &dyn StateView,
    txn: &mut SignedUserTransactionView,
) -> anyhow::Result<()> {
    let txn_payload = bcs_ext::from_bytes(txn.raw_txn.payload.0.as_slice())?;
    match decode_txn_payload(state, &txn_payload) {
        // ignore decode failure, as txns may has invalid payload here.
        Err(e) => {
            debug!(
                "decode payload of txn {} failure, {:?}",
                txn.transaction_hash, e
            );
        }
        Ok(d) => {
            txn.raw_txn.decoded_payload = Some(d.into());
        }
    }
    Ok(())
}
