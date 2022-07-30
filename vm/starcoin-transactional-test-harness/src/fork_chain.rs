// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, bail, ensure, format_err, Result};
use dashmap::DashMap;
use jsonrpc_client_transports::RpcError;
use jsonrpc_core::futures_util::{FutureExt, TryFutureExt};
use starcoin_chain_api::{
    ChainReader,
};
use starcoin_rpc_api::FutureResult;
use starcoin_config::{ChainNetworkID, BuiltinNetworkID};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::{ChainApi, GetBlockOption};
use starcoin_rpc_api::state::StateApi;
use starcoin_rpc_api::types::{
    BlockInfoView, BlockView, ChainId, ChainInfoView, StrView,
};
use starcoin_rpc_server::module::map_err;
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_types::block::BlockHeaderBuilder;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockInfo, BlockNumber},
};
use starcoin_vm_types::access_path::AccessPath;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::option::Option::{None, Some};
use std::sync::{Arc, Mutex};
use crate::remote_state::RemoteRpcAsyncClient;
use starcoin_accumulator::{
    node::AccumulatorStoreType, Accumulator, MerkleAccumulator,
};
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{BlockStore, Storage, Store};

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct ChainStatusWithBlock {
    pub status: ChainStatus,
    pub head: Block,
}

#[derive(Clone)]
pub struct ForkBlockChain {    
    remote_viewer: Arc<RemoteRpcAsyncClient>,
    storage: Arc<Storage>,
    fork_number: u64,
    current_number: u64,
    status: Option<ChainStatusWithBlock>,
    block_map: DashMap<HashValue, Block>,
    number_hash_map: DashMap<u64, HashValue>,
}

impl ForkBlockChain {
    pub fn new(remote_viewer: Arc<RemoteRpcAsyncClient>, fork_number: u64) -> Result<Self> {
        let storage_instance = StorageInstance::new_cache_instance();
        let storage = Arc::new(Storage::new(storage_instance)?);

        Ok(Self {
            remote_viewer,
            storage,
            fork_number,
            current_number: fork_number,
            status: None,
            block_map: DashMap::new(),
            number_hash_map: DashMap::new(),
        })
    }

    pub fn get_remote_viewer(&self) -> Arc<RemoteRpcAsyncClient> {
        self.remote_viewer.clone()
    }

    pub fn add_new_block(&mut self, mut block: Block) -> Result<()> {
        block.header = block.header().as_builder().build();

        let txn_accumulator = MerkleAccumulator::new_empty(
            self.storage.get_accumulator_store(AccumulatorStoreType::Transaction),
        );
        let block_accumulator = MerkleAccumulator::new_empty(
            self.storage.get_accumulator_store(AccumulatorStoreType::Block),
        );
        let block_info = BlockInfo::new(
            block.header.id(),
            block.header.difficulty(),
            txn_accumulator.get_info(),
            block_accumulator.get_info(),
        );      
        self.current_number = block.header().number();
        self.number_hash_map.insert(self.current_number, block.header().id());
        self.block_map.insert(block.header().id(), block.clone());
        self.status = Some(ChainStatusWithBlock {
            status: ChainStatus::new(block.header().clone(), block_info.clone()),
            head: block.clone(),
        });
        self.storage.save_block_info(block_info)?;
        self.storage.commit_block(block)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct MockApi {
    pub chain: Arc<Mutex<ForkBlockChain>>,
}

impl MockApi {
    pub fn new(chain: Arc<Mutex<ForkBlockChain>>) -> Self {
        Self { chain }
    }
}

impl ChainApi for MockApi
{
    fn id(&self) -> jsonrpc_core::Result<ChainId> {
        Ok(ChainId::from(&ChainNetworkID::Builtin(BuiltinNetworkID::Dev)))
    }

    fn info(&self) -> FutureResult<ChainInfoView> {
        let chain = self.chain.lock().unwrap();
        let status = chain.status.clone();        
        let client = chain.remote_viewer.get_chain_client().clone();
        let fut = async move {
            match status {
                Some(status) => {
                    Ok(ChainInfoView::from(
                        ChainInfo::new(
                            status.head.header().chain_id(),
                            HashValue::random(),
                            status.status.clone(),
                        )
                    ))
                },
                None => {                
                    client
                        .info()
                        .await
                        .map_err(|e| anyhow!("{}", e))
                }
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
        let client = chain.remote_viewer.get_chain_client().clone();
        let fut = async move {
            match status {
                Some(status) => {
                    todo!()
                },
                None => {                
                    client
                        .get_block_by_hash(hash, option)
                        .await
                        .map_err(|e| anyhow!("{}", e))
                }
            }
        };
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_block_by_number(
        &self,
        number:BlockNumber,
        option:Option<GetBlockOption>,
    ) -> FutureResult<Option<BlockView>> {
        let chain = self.chain.lock().unwrap();
        let status = chain.status.clone();        
        let client = chain.remote_viewer.get_chain_client().clone();
        let fork_number = chain.fork_number;
        let current_number = chain.current_number;
        let number_hash_map = chain.number_hash_map.clone();
        let block_map = chain.block_map.clone();
        let fut = async move {
            if number <= fork_number {
                client
                .get_block_by_number(number, option)
                .await
                .map_err(|e| anyhow!("{}", e))
            } else if number <= current_number {
                debug_assert!(status.is_some());
                let hash = number_hash_map.get(&number).map(|h| *h);
                let block_view = match hash {
                    Some(hash) => {
                        match block_map.get(&hash) {
                            Some(b) => Some(BlockView::try_from(b.clone())?),
                            None => None,
                        }
                    },
                    None => None,
                };
                Ok(block_view)
            } else {
                Ok(None)
            }
        };
        println!("run in get block by number: {}, current number: {}", number, current_number);
        Box::pin(fut.boxed().map_err(map_err))
    }

    fn get_blocks_by_number(
        &self,
        number:Option<BlockNumber>,
        count:u64,
    ) -> FutureResult<Vec<BlockView>>  {
        todo!()
    }

    fn get_block_info_by_number(
        &self,
        number:BlockNumber
    ) -> FutureResult<Option<BlockInfoView>>  {
        let chain = self.chain.lock().unwrap();
        let status = chain.status.clone();        
        let client = Arc::new(chain.remote_viewer.get_chain_client());
        let fut = async move {
            match status {
                Some(status) => {
                    todo!()
                },
                None => {                
                    client
                        .get_block_info_by_number(number)
                        .await
                }
            }
        };
        // Box::pin(fut.boxed())
        todo!()
    }

    fn get_transaction(&self,transaction_hash:HashValue,option:Option<starcoin_rpc_api::chain::GetTransactionOption> ,) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::TransactionView> >  {
        todo!()
    }

    fn get_transaction_info(&self,transaction_hash:HashValue,) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::TransactionInfoView> >  {
        todo!()
    }

    fn get_block_txn_infos(&self,block_hash:HashValue) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::TransactionInfoView> >  {
        todo!()
    }

    fn get_txn_info_by_block_and_index(&self,block_hash:HashValue,idx:u64,) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::TransactionInfoView> >  {
        todo!()
    }

    fn get_events_by_txn_hash(&self,txn_hash:HashValue,option:Option<starcoin_rpc_api::chain::GetEventOption> ,) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::TransactionEventResponse> >  {
        todo!()
    }

    fn get_events(&self,filter:starcoin_rpc_api::types::pubsub::EventFilter,option:Option<starcoin_rpc_api::chain::GetEventOption> ,) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::TransactionEventResponse> >  {
        todo!()
    }

    fn get_headers(&self,ids:Vec<HashValue>) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::BlockHeaderView> >  {
        todo!()
    }

    fn get_transaction_infos(&self,start_global_index:u64,reverse:bool,max_size:u64,) -> starcoin_rpc_api::FutureResult<Vec<starcoin_rpc_api::types::TransactionInfoView> >  {
        todo!()
    }

    fn get_transaction_proof(&self,block_hash:HashValue,transaction_global_index:u64,event_index:Option<u64> ,access_path:Option<starcoin_rpc_api::types::StrView<AccessPath> > ,) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::TransactionInfoWithProofView> >  {
        todo!()
    }

    fn get_transaction_proof_raw(&self,block_hash:HashValue,transaction_global_index:u64,event_index:Option<u64> ,access_path:Option<starcoin_rpc_api::types::StrView<AccessPath> > ,) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::StrView<Vec<u8> > > >  {
        todo!()
    }
}

impl StateApi for MockApi 
{
    fn get(&self, access_path: AccessPath) -> starcoin_rpc_api::FutureResult<Option<Vec<u8>>> {
        todo!()
    }

    fn get_with_proof(&self, access_path: AccessPath) -> starcoin_rpc_api::FutureResult<starcoin_rpc_api::types::StateWithProofView> {
        todo!()
    }

    fn get_with_proof_raw(&self, access_path: AccessPath) -> starcoin_rpc_api::FutureResult<StrView<Vec<u8>>> {
        todo!()
    }

    fn get_account_state(&self, address: AccountAddress) -> starcoin_rpc_api::FutureResult<Option<starcoin_types::account_state::AccountState>> {
        todo!()
    }

    fn get_account_state_set(
        &self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::AccountStateSetView>> {
        todo!()
    }

    fn get_state_root(&self) -> starcoin_rpc_api::FutureResult<HashValue> {
        todo!()
    }

    fn get_with_proof_by_root(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> starcoin_rpc_api::FutureResult<starcoin_rpc_api::types::StateWithProofView> {
        todo!()
    }

    fn get_with_proof_by_root_raw(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> starcoin_rpc_api::FutureResult<StrView<Vec<u8>>> {
        todo!()
    }

    fn get_code(
        &self,
        module_id: StrView<move_core_types::language_storage::ModuleId>,
        option: Option<starcoin_rpc_api::state::GetCodeOption>,
    ) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::CodeView>> {
        todo!()
    }

    fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StrView<move_core_types::language_storage::StructTag>,
        option: Option<starcoin_rpc_api::state::GetResourceOption>,
    ) -> starcoin_rpc_api::FutureResult<Option<starcoin_rpc_api::types::ResourceView>> {
        todo!()
    }

    fn list_resource(
        &self,
        addr: AccountAddress,
        option: Option<starcoin_rpc_api::state::ListResourceOption>,
    ) -> starcoin_rpc_api::FutureResult<starcoin_rpc_api::types::ListResourceView> {
        todo!()
    }

    fn list_code(
        &self,
        addr: AccountAddress,
        option: Option<starcoin_rpc_api::state::ListCodeOption>,
    ) -> starcoin_rpc_api::FutureResult<starcoin_rpc_api::types::ListCodeView> {
        todo!()
    }
}
