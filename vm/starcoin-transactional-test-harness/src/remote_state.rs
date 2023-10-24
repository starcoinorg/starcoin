// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use jsonrpc_client_transports::RpcChannel;
use move_binary_format::errors::VMError;
use move_core_types::resolver::{ModuleResolver, ResourceResolver};
use starcoin_crypto::HashValue;

use move_table_extension::{TableHandle, TableResolver};
use starcoin_rpc_api::chain::ChainApiClient;
use starcoin_rpc_api::state::StateApiClient;
use starcoin_rpc_api::types::{BlockView, StateWithProofView, StateWithTableItemProofView};
use starcoin_state_api::ChainStateWriter;
use starcoin_types::access_path::{AccessPath, DataPath};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag};
use starcoin_types::state_set::ChainStateSet;
use starcoin_types::vm_error::StatusCode;
use starcoin_vm_types::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::table::TableHandle as StarcoinTableHandle;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::write_set::WriteSet;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub enum SelectableStateView<A, B> {
    A(A),
    B(B),
}

impl<A, B> ModuleResolver for SelectableStateView<A, B>
where
    A: ModuleResolver,
    B: ModuleResolver<Error = A::Error>,
{
    type Error = A::Error;

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        match self {
            Self::A(a) => a.get_module(module_id),
            Self::B(b) => b.get_module(module_id),
        }
    }
}
impl<A, B> ResourceResolver for SelectableStateView<A, B>
where
    A: ResourceResolver,
    B: ResourceResolver<Error = A::Error>,
{
    type Error = A::Error;
    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        match self {
            Self::A(v) => v.get_resource(address, tag),
            Self::B(v) => v.get_resource(address, tag),
        }
    }
}
impl<A, B> StateView for SelectableStateView<A, B>
where
    A: StateView,
    B: StateView,
{
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<Vec<u8>>> {
        match self {
            SelectableStateView::A(a) => a.get_state_value(state_key),
            SelectableStateView::B(b) => b.get_state_value(state_key),
        }
    }

    fn is_genesis(&self) -> bool {
        false
    }

    fn get_block_number(&self) -> Option<u64> {
        None
    }
}
impl<A, B> ChainStateWriter for SelectableStateView<A, B>
where
    A: ChainStateWriter,
    B: ChainStateWriter,
{
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        match self {
            SelectableStateView::A(a) => a.set(access_path, value),
            SelectableStateView::B(b) => b.set(access_path, value),
        }
    }

    fn remove(&self, access_path: &AccessPath) -> Result<()> {
        match self {
            SelectableStateView::A(a) => a.remove(access_path),
            SelectableStateView::B(b) => b.remove(access_path),
        }
    }

    fn apply(&self, state_set: ChainStateSet) -> Result<()> {
        match self {
            SelectableStateView::A(a) => a.apply(state_set),
            SelectableStateView::B(b) => b.apply(state_set),
        }
    }

    fn apply_write_set(&self, write_set: WriteSet) -> Result<()> {
        match self {
            SelectableStateView::A(a) => a.apply_write_set(write_set),
            SelectableStateView::B(b) => b.apply_write_set(write_set),
        }
    }

    fn commit(&self) -> Result<HashValue> {
        match self {
            SelectableStateView::A(a) => a.commit(),
            SelectableStateView::B(b) => b.commit(),
        }
    }

    fn flush(&self) -> Result<()> {
        match self {
            SelectableStateView::A(a) => a.flush(),
            SelectableStateView::B(b) => b.flush(),
        }
    }
}

pub struct UnionedRemoteCache<A, B> {
    pub a: A,
    pub b: B,
}

impl<A, B> UnionedRemoteCache<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A, B> ResourceResolver for UnionedRemoteCache<A, B>
where
    A: ResourceResolver,
    B: ResourceResolver<Error = A::Error>,
{
    type Error = A::Error;
    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        match self.a.get_resource(address, tag)? {
            Some(d) => Ok(Some(d)),
            None => self.b.get_resource(address, tag),
        }
    }
}
impl<A, B> ModuleResolver for UnionedRemoteCache<A, B>
where
    A: ModuleResolver,
    B: ModuleResolver<Error = A::Error>,
{
    type Error = A::Error;

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        match self.a.get_module(module_id)? {
            Some(d) => Ok(Some(d)),
            None => self.b.get_module(module_id),
        }
    }
}
impl<A, B> StateView for UnionedRemoteCache<A, B>
where
    A: StateView,
    B: StateView,
{
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<Vec<u8>>> {
        match self.a.get_state_value(state_key)? {
            None => self.b.get_state_value(state_key),
            Some(d) => Ok(Some(d)),
        }
    }

    fn is_genesis(&self) -> bool {
        false
    }

    fn get_block_number(&self) -> Option<u64> {
        None
    }
}

//TODO migrate this to rpc client crate.
#[derive(Clone)]
pub struct RemoteRpcAsyncClient {
    state_client: StateApiClient,
    chain_client: ChainApiClient,
    state_root: HashValue,
    fork_number: u64,
    fork_block_hash: HashValue,
}

impl RemoteRpcAsyncClient {
    pub async fn from_url(rpc_url: &str, block_number: Option<u64>) -> Result<Self> {
        let rpc_channel: RpcChannel = jsonrpc_client_transports::transports::http::connect(rpc_url)
            .await
            .map_err(|e| anyhow!(format!("{}", e)))?;
        let chain_client: starcoin_rpc_api::chain::ChainApiClient = rpc_channel.clone().into();
        let (state_root, fork_number, fork_block_hash) = match block_number {
            None => {
                let chain_info = chain_client
                    .info()
                    .await
                    .map_err(|e| anyhow!(format!("{}", e)))?;
                (
                    chain_info.head.state_root,
                    chain_info.head.number.0,
                    chain_info.head.block_hash,
                )
            }
            Some(n) => {
                let b: Option<BlockView> = chain_client
                    .get_block_by_number(n, None)
                    .await
                    .map_err(|e| anyhow!(format!("{}", e)))?;
                let b = b.ok_or_else(|| anyhow::anyhow!("cannot found block of height {}", n))?;
                (b.header.state_root, n, b.header.block_hash)
            }
        };
        let state_client: starcoin_rpc_api::state::StateApiClient = rpc_channel.clone().into();
        Ok(Self {
            state_client,
            chain_client,
            state_root,
            fork_number,
            fork_block_hash,
        })
    }

    pub async fn get_modules_async(
        &self,
        addr: AccountAddress,
    ) -> VMResult<Option<BTreeMap<Identifier, Vec<u8>>>> {
        let state = self
            .state_client
            .get_account_state_set(addr, Some(self.state_root))
            .await
            .map_err(|_| {
                PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined)
            })?;
        Ok(match state {
            None => None,
            Some(account_state_set) => Some(
                account_state_set
                    .codes
                    .into_iter()
                    .map(|(k, c)| (k, c.0.to_vec()))
                    .collect(),
            ),
        })
    }

    pub async fn get_module_async(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        let ap = AccessPath::new(
            *module_id.address(),
            DataPath::Code(module_id.name().to_owned()),
        );
        let state_with_proof: StateWithProofView = self
            .state_client
            .get_with_proof_by_root(ap, self.state_root)
            .await
            .map_err(|_| {
                PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined)
            })?;
        Ok(state_with_proof.state.map(|v| v.0))
    }

    pub async fn get_resource_async(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        let ap = AccessPath::new(*address, DataPath::Resource(tag.clone()));
        let state_with_proof = self
            .state_client
            .get_with_proof_by_root(ap, self.state_root)
            .await
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR))?;
        Ok(state_with_proof.state.map(|v| v.0))
    }
    pub async fn resolve_table_entry_async(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        let handle1: StarcoinTableHandle = StarcoinTableHandle(handle.0);
        let state_table_item_proof: StateWithTableItemProofView = self
            .state_client
            .get_with_table_item_proof_by_root(handle1, key.to_vec(), self.state_root)
            .await
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR))?;
        Ok(state_table_item_proof.key_proof.0.map(|v| v.0))
    }
    pub fn get_chain_client(&self) -> &ChainApiClient {
        &self.chain_client
    }

    pub fn get_state_client(&self) -> &StateApiClient {
        &self.state_client
    }

    pub fn get_fork_block_number(&self) -> u64 {
        self.fork_number
    }

    pub fn get_fork_state_root(&self) -> HashValue {
        self.state_root
    }

    pub fn get_fork_block_hash(&self) -> HashValue {
        self.fork_block_hash
    }
}

#[derive(Clone)]
pub struct RemoteViewer {
    svc: Arc<RemoteRpcAsyncClient>,
    rt: Arc<Runtime>,
}

impl RemoteViewer {
    pub fn from_url(rpc_url: &str, block_number: Option<u64>) -> Result<Self> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .thread_name("remote-state-worker")
            .enable_all()
            .build()?;

        let v =
            rt.block_on(async { RemoteRpcAsyncClient::from_url(rpc_url, block_number).await })?;

        Ok(Self {
            svc: Arc::new(v),
            rt: Arc::new(rt),
        })
    }

    pub fn new(rpc_async_client: Arc<RemoteRpcAsyncClient>, rt: Arc<Runtime>) -> Self {
        Self {
            svc: rpc_async_client,
            rt,
        }
    }

    pub fn get_modules(
        &self,
        addr: AccountAddress,
    ) -> VMResult<Option<BTreeMap<Identifier, Vec<u8>>>> {
        let handle = self.rt.handle().clone();
        handle.block_on(self.svc.get_modules_async(addr))
    }
}

impl ModuleResolver for RemoteViewer {
    type Error = VMError;

    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        let handle = self.rt.handle().clone();
        handle.block_on(self.svc.get_module_async(module_id))
    }
}

impl ResourceResolver for RemoteViewer {
    type Error = PartialVMError;
    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        let handle = self.rt.handle().clone();
        handle.block_on(self.svc.get_resource_async(address, tag))
    }
}

impl TableResolver for RemoteViewer {
    fn resolve_table_entry(&self, handle: &TableHandle, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let h = self.rt.handle().clone();
        h.block_on(self.svc.resolve_table_entry_async(handle, key))
    }
}

impl StateView for RemoteViewer {
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<Vec<u8>>> {
        match state_key {
            StateKey::AccessPath(access_path) => match &access_path.path {
                DataPath::Code(m) => Ok(self
                    .get_module(&ModuleId::new(access_path.address, m.clone()))
                    .map_err(|err| err.into_vm_status())?),
                DataPath::Resource(s) => Ok(self
                    .get_resource(&access_path.address, s)
                    .map_err(|err| err.finish(Location::Undefined).into_vm_status())?),
            },
            StateKey::TableItem(table_item) => Ok(self.resolve_table_entry(
                &move_table_extension::TableHandle(table_item.handle.0),
                table_item.key.as_slice(),
            )?),
        }
    }

    fn is_genesis(&self) -> bool {
        false
    }

    fn get_block_number(&self) -> Option<u64> {
        None
    }
}
