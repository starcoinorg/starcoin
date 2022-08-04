// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::remote_state::RemoteRpcAsyncClient;
use anyhow::{anyhow, bail, Result};
use dashmap::DashMap;
use futures::executor::block_on;
use jsonrpc_core::futures_util::{FutureExt, TryFutureExt};
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
use starcoin_config::{BuiltinNetworkID, ChainNetworkID};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::ChainApiClient;
use starcoin_rpc_api::chain::{ChainApi, GetBlockOption};
use starcoin_rpc_api::types::{BlockInfoView, BlockView, ChainId, ChainInfoView};
use starcoin_rpc_api::FutureResult;
use starcoin_rpc_server::module::map_err;
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{BlockStore, Storage, Store};
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::transaction::{TransactionOutput, TransactionInfo};
use starcoin_types::{
    block::{Block, BlockInfo, BlockNumber},
};
use starcoin_vm_types::access_path::AccessPath;
use std::hash::Hash;
use std::option::Option::{None, Some};
use std::sync::{Arc, Mutex};

pub struct MockBlockInfoStore {
    remote: Arc<ChainApiClient>,
    store: Arc<dyn BlockInfoStore>,
}

impl MockBlockInfoStore {
    pub fn new(chain_client: Arc<ChainApiClient>, store: Arc<dyn BlockInfoStore>) -> Self {
        Self { remote: chain_client, store }
    }
}

impl BlockInfoStore for MockBlockInfoStore {
    fn save_block_info(&self, block_info: BlockInfo) -> Result<()> {
        self.store.save_block_info(block_info)
    }

    fn get_block_info(&self, hash_value: HashValue) -> Result<Option<BlockInfo>> {
        let block_info = match self.store.get_block_info(hash_value)? {
            Some(block_info) => Some(block_info),
            None => {
                let block_view = 
                    block_on(self.remote.get_block_by_hash(hash_value, 
                        Some(GetBlockOption { decode: true, raw: false })))
                    .map_err(|e| anyhow!("{}", e))?;
                let block_info: Option<BlockInfo> = match block_view {
                    Some(block_view) => {
                        block_on(self.remote.get_block_info_by_number(block_view.header.number.0))
                            .map_err(|e| anyhow!("{}", e))?
                            .map(|view| view.into())
                    },
                    None => None,
                };
                block_info
            }
        };
        Ok(block_info)
    }

    fn delete_block_info(&self, block_hash: HashValue) -> Result<()> {
        self.store.delete_block_info(block_hash)
    }

    fn get_block_infos(&self, ids: Vec<HashValue>) -> Result<Vec<Option<BlockInfo>>> {
        todo!()
    }
}


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
    block_map: DashMap<HashValue, Block>,
    number_hash_map: DashMap<u64, HashValue>,
    txn_accumulator: MerkleAccumulator,
    state_root: Arc<Mutex<HashValue>>,
}

impl ForkBlockChain {
    pub fn new(state_root: Arc<Mutex<HashValue>>) -> Result<Self> {
        Self::new_inner(0, None, state_root)
    }

    pub fn fork(remote_client: Arc<RemoteRpcAsyncClient>, fork_number: u64, state_root: Arc<Mutex<HashValue>>) -> Result<Self> {
        Self::new_inner(fork_number, Some(remote_client), state_root)
    }
    // Mock chain fork from remote_client if fork_number > 0
    fn new_inner(
        fork_number: u64,
        remote_client: Option<Arc<RemoteRpcAsyncClient>>,
        state_root: Arc<Mutex<HashValue>>,
    ) -> Result<Self> {
        let storage_instance = StorageInstance::new_cache_instance();
        let storage = Arc::new(Storage::new(storage_instance)?);

        let accumulator_store = 
            storage.get_accumulator_store(AccumulatorStoreType::Transaction);
        let txn_accumulator = match remote_client.clone() {
            Some(client) => {
                let block_info: Option<BlockInfo> = block_on(
                    client
                    .get_chain_client()
                    .get_block_info_by_number(fork_number))
                    .map_err(|e| anyhow!("{}", e))?
                    .map(|view| view.into());
                match block_info {
                    Some(block) => MerkleAccumulator::new_with_info(block.txn_accumulator_info, accumulator_store),
                    None => MerkleAccumulator::new_empty(accumulator_store),
                }
            },
            None => MerkleAccumulator::new_empty(accumulator_store),
        };
        Ok(Self {
            remote_client,
            storage,
            fork_number,
            current_number: fork_number,
            status: None,
            block_map: DashMap::new(),
            number_hash_map: DashMap::new(),
            txn_accumulator,
            state_root,
        })
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
        self.number_hash_map
            .insert(self.current_number, block.header().id());
        self.block_map.insert(block.header().id(), block.clone());
        self.status = Some(ChainStatusWithBlock {
            status: ChainStatus::new(block.header().clone(), block_info.clone()),
            head: block.clone(),
        });
        self.storage.save_block_info(block_info)?;
        self.storage.commit_block(block)?;
        Ok(())
    }

    pub fn add_new_txn(&mut self, txn_hash: HashValue, output: TransactionOutput) -> Result<()> {
        let state_root = *self.state_root.lock().unwrap();
        let (_, events, gas_used, status) = output.into_inner();
        let status = status
            .status()
            .expect("TransactionStatus at here must been KeptVMStatus");
        let txn_info = TransactionInfo::new(
            txn_hash,
            state_root,
            events.as_slice(),
            gas_used,
            status,
        );
        self.txn_accumulator.append(&[txn_info.id()])?;
        Ok(())
    }

    pub fn txn_accumulator_root(&self) -> HashValue {
        self.txn_accumulator.root_hash()
    }

    fn remote_chain_client(&self) -> Option<ChainApiClient> {
        match self.remote_client.clone() {
            Some(client) => Some(client.get_chain_client().clone()),
            None => None,
        }
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
        let client = chain.remote_chain_client().clone();
        let fut = async move {
            match status {
                Some(status) => Ok(ChainInfoView::from(ChainInfo::new(
                    status.head.header().chain_id(),
                    HashValue::random(),
                    status.status.clone(),
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
        let status = chain.status.clone();
        let client = chain.remote_chain_client().clone();
        let fut = async move {
            match status {
                Some(status) => {
                    todo!()
                }
                None => match client {
                    Some(client) => client
                        .get_block_by_hash(hash, option)
                        .await
                        .map_err(|e| anyhow!("{}", e)),
                    None => bail!("No block generated."),
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
        let status = chain.status.clone();
        let client = chain.remote_chain_client();
        let fork_number = chain.fork_number;
        let current_number = chain.current_number;
        let number_hash_map = chain.number_hash_map.clone();
        let block_map = chain.block_map.clone();
        let fut = async move {
            if number <= fork_number {
                debug_assert!(client.is_some());
                client
                    .unwrap()
                    .get_block_by_number(number, option)
                    .await
                    .map_err(|e| anyhow!("{}", e))
            } else if number <= current_number {
                debug_assert!(status.is_some());
                let hash = number_hash_map.get(&number).map(|h| *h);
                let block_view = match hash {
                    Some(hash) => match block_map.get(&hash) {
                        Some(b) => Some(BlockView::try_from(b.clone())?),
                        None => None,
                    },
                    None => None,
                };
                Ok(block_view)
            } else {
                Ok(None)
            }
        };
        println!(
            "run in get block by number: {}, current number: {}",
            number, current_number
        );
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> FutureResult<Vec<BlockView>> {
        todo!()
    }

    fn get_block_info_by_number(&self, number: BlockNumber) -> FutureResult<Option<BlockInfoView>> {
        let chain = self.chain.lock().unwrap();
        let status = chain.status.clone();
        let client = chain.remote_chain_client();
        let fut = async move {
            match status {
                Some(status) => {
                    todo!()
                }
                None => {
                    debug_assert!(client.is_some());
                    client.unwrap().get_block_info_by_number(number).await
                }
            }
        };
        // Box::pin(fut.boxed())
        todo!()
    }

    fn get_transaction(
        &self,
        transaction_hash: HashValue,
        option: Option<starcoin_rpc_api::chain::GetTransactionOption>,
    ) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::TransactionView>> {
        todo!()
    }

    fn get_transaction_info(
        &self,
        transaction_hash: HashValue,
    ) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::TransactionInfoView>> {
        todo!()
    }

    fn get_block_txn_infos(
        &self,
        block_hash: HashValue,
    ) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::TransactionInfoView>> {
        todo!()
    }

    fn get_txn_info_by_block_and_index(
        &self,
        block_hash: HashValue,
        idx: u64,
    ) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::TransactionInfoView>> {
        todo!()
    }

    fn get_events_by_txn_hash(
        &self,
        txn_hash: HashValue,
        option: Option<starcoin_rpc_api::chain::GetEventOption>,
    ) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::TransactionEventResponse>>
    {
        todo!()
    }

    fn get_events(
        &self,
        filter: starcoin_rpc_api::types::pubsub::EventFilter,
        option: Option<starcoin_rpc_api::chain::GetEventOption>,
    ) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::TransactionEventResponse>>
    {
        todo!()
    }

    fn get_headers(
        &self,
        ids: Vec<HashValue>,
    ) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::BlockHeaderView>> {
        todo!()
    }

    fn get_transaction_infos(
        &self,
        start_global_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::TransactionInfoView>> {
        todo!()
    }

    fn get_transaction_proof(
        &self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<starcoin_rpc_api::types::StrView<AccessPath>>,
    ) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::TransactionInfoWithProofView>>
    {
        todo!()
    }

    fn get_transaction_proof_raw(
        &self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<starcoin_rpc_api::types::StrView<AccessPath>>,
    ) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::StrView<Vec<u8>>>> {
        todo!()
    }
}
