// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use jsonrpc_client_transports::RpcChannel;
use move_binary_format::errors::VMError;
use starcoin_crypto::HashValue;

use jsonrpc_http_server::hyper::body::Bytes;
use move_core_types::metadata::Metadata;
use move_core_types::resolver::{ModuleResolver, ResourceResolver};
use move_core_types::value::MoveTypeLayout;
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
use starcoin_vm_types::state_store::errors::StateviewError;
use starcoin_vm_types::state_store::state_key::inner::StateKeyInner;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::state_storage_usage::StateStorageUsage;
use starcoin_vm_types::state_store::state_value::StateValue;
use starcoin_vm_types::state_store::table::TableHandle as StarcoinTableHandle;
use starcoin_vm_types::state_store::TStateView;
use starcoin_vm_types::state_view::StateReaderExt;
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

    fn get_module_metadata(&self, module_id: &ModuleId) -> Vec<Metadata> {
        match self {
            Self::A(a) => a.get_module_metadata(module_id),
            Self::B(b) => b.get_module_metadata(module_id),
        }
    }

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Bytes>, Self::Error> {
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
    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        metadata: &[Metadata],
        layout: Option<&MoveTypeLayout>,
    ) -> std::result::Result<(Option<Bytes>, usize), Self::Error> {
        match self {
            Self::A(v) => {
                v.get_resource_bytes_with_metadata_and_layout(address, struct_tag, metadata, layout)
            }
            Self::B(v) => {
                v.get_resource_bytes_with_metadata_and_layout(address, struct_tag, metadata, layout)
            }
        }
    }
}
impl<A, B> TStateView for SelectableStateView<A, B>
where
    A: TStateView<Key = StateKey>,
    B: TStateView<Key = StateKey>,
{
    type Key = StateKey;

    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<StateValue>, StateviewError> {
        match self {
            Self::A(a) => a.get_state_value(state_key),
            Self::B(b) => b.get_state_value(state_key),
        }
    }

    fn get_usage(&self) -> starcoin_vm_types::state_store::Result<StateStorageUsage> {
        unimplemented!("not implemented")
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
impl<A, B> ChainStateWriter for SelectableStateView<A, B>
where
    A: ChainStateWriter,
    B: ChainStateWriter,
{
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        match self {
            Self::A(a) => a.set(access_path, value),
            Self::B(b) => b.set(access_path, value),
        }
    }

    fn remove(&self, access_path: &AccessPath) -> Result<()> {
        match self {
            Self::A(a) => a.remove(access_path),
            Self::B(b) => b.remove(access_path),
        }
    }

    fn apply(&self, state_set: ChainStateSet) -> Result<()> {
        match self {
            Self::A(a) => a.apply(state_set),
            Self::B(b) => b.apply(state_set),
        }
    }

    fn apply_write_set(&self, write_set: WriteSet) -> Result<()> {
        match self {
            Self::A(a) => a.apply_write_set(write_set),
            Self::B(b) => b.apply_write_set(write_set),
        }
    }

    fn commit(&self) -> Result<HashValue> {
        match self {
            Self::A(a) => a.commit(),
            Self::B(b) => b.commit(),
        }
    }

    fn flush(&self) -> Result<()> {
        match self {
            Self::A(a) => a.flush(),
            Self::B(b) => b.flush(),
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

    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        metadata: &[Metadata],
        layout: Option<&MoveTypeLayout>,
    ) -> std::result::Result<(Option<Bytes>, usize), Self::Error> {
        self.a
            .get_resource_bytes_with_metadata_and_layout(address, struct_tag, metadata, layout)
            .or_else(|_| {
                self.b.get_resource_bytes_with_metadata_and_layout(
                    address, struct_tag, metadata, layout,
                )
            })
    }
}
impl<A, B> ModuleResolver for UnionedRemoteCache<A, B>
where
    A: ModuleResolver,
    B: ModuleResolver<Error = A::Error>,
{
    type Error = A::Error;

    fn get_module_metadata(&self, module_id: &ModuleId) -> Vec<Metadata> {
        match self.a.get_module_metadata(module_id) {
            d if !d.is_empty() => d,
            _ => self.b.get_module_metadata(module_id),
        }
    }
    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Bytes>, Self::Error> {
        match self.a.get_module(module_id)? {
            Some(d) => Ok(Some(d)),
            None => self.b.get_module(module_id),
        }
    }
}
impl<A, B> TStateView for UnionedRemoteCache<A, B>
where
    A: TStateView<Key = StateKey>,
    B: TStateView<Key = StateKey>,
{
    type Key = StateKey;

    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<StateValue>, StateviewError> {
        match self.a.get_state_value(state_key)? {
            None => self.b.get_state_value(state_key),
            Some(d) => Ok(Some(d)),
        }
    }

    fn get_usage(&self) -> starcoin_vm_types::state_store::Result<StateStorageUsage> {
        unimplemented!("get_usage not implemented")
    }

    fn is_genesis(&self) -> bool {
        false
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
        let state_key = StateKey::module_id(module_id);
        let state_with_proof: StateWithProofView = self
            .state_client
            .get_with_proof_by_root(state_key, self.state_root)
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
        let state_key = StateKey::resource(address, tag);
        let state_with_proof = self
            .state_client
            .get_with_proof_by_root(state_key, self.state_root)
            .await
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR))?;
        Ok(state_with_proof.state.map(|v| v.0))
    }
    pub async fn resolve_table_entry_async(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> PartialVMResult<Option<Vec<u8>>> {
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

    fn get_module_metadata(&self, _module_id: &ModuleId) -> Vec<Metadata> {
        todo!()
    }

    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Bytes>> {
        let handle = self.rt.handle().clone();
        let bytes = handle.block_on(self.svc.get_module_async(module_id))?;
        Ok(bytes.map(Into::into))
    }
}

impl ResourceResolver for RemoteViewer {
    type Error = PartialVMError;

    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        _metadata: &[Metadata],
        _layout: Option<&MoveTypeLayout>,
    ) -> PartialVMResult<(Option<Bytes>, usize)> {
        let handle = self.rt.handle().clone();
        let bytes = handle.block_on(self.svc.get_resource_async(address, struct_tag))?;
        Ok(bytes.map_or((None, 0), |r| {
            let b = Bytes::from(r);
            let len = b.len();
            (Some(b), len)
        }))
    }
}

impl TableResolver for RemoteViewer {
    fn resolve_table_entry_bytes_with_layout(
        &self,
        handle: &TableHandle,
        key: &[u8],
        _maybe_layout: Option<&MoveTypeLayout>,
    ) -> std::result::Result<Option<Bytes>, PartialVMError> {
        let h = self.rt.handle().clone();
        let bytes = h.block_on(self.svc.resolve_table_entry_async(handle, key))?;
        Ok(bytes.map(Into::into))
    }
}

impl TStateView for RemoteViewer {
    type Key = StateKey;

    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<StateValue>, StateviewError> {
        match state_key.inner() {
            StateKeyInner::AccessPath(access_path) => match &access_path.path {
                DataPath::Code(m) => Ok(self
                    .get_module(&ModuleId::new(access_path.address, m.clone()))
                    .map_err(|_| StateviewError::Other("get_module error".to_string()))?
                    .map(StateValue::from)),
                DataPath::Resource(s) => {
                    let ret = self
                        .get_resource(access_path.address, s)
                        .map_err(|_| StateviewError::Other("get_resource error".to_string()))?;
                    Ok(Some(StateValue::from(ret)))
                }
                _ => unimplemented!("todo"),
            },
            StateKeyInner::TableItem { handle, key } => Ok(self
                .resolve_table_entry_bytes_with_layout(&TableHandle(handle.0), key, None)
                .map_err(|_| StateviewError::Other("table_item".to_string()))?
                .map(StateValue::from)),
            _ => todo!(),
        }
    }

    fn get_usage(&self) -> starcoin_vm_types::state_store::Result<StateStorageUsage> {
        todo!()
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
