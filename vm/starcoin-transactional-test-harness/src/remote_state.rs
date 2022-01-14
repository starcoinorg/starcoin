use anyhow::{anyhow, Result};
use jsonrpc_client_transports::RpcChannel;
use move_binary_format::errors::VMError;
use move_core_types::resolver::{ModuleResolver, ResourceResolver};
use starcoin_crypto::HashValue;

use starcoin_rpc_api::state::StateApiClient;
use starcoin_rpc_api::types::{BlockView, StateWithProofView};
use starcoin_state_api::ChainStateWriter;
use starcoin_types::access_path::{AccessPath, DataPath};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag};
use starcoin_types::state_set::ChainStateSet;
use starcoin_types::vm_error::StatusCode;
use starcoin_vm_types::errors::{Location, PartialVMError, PartialVMResult, VMResult};
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
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        match self {
            SelectableStateView::A(a) => a.get(access_path),
            SelectableStateView::B(b) => b.get(access_path),
        }
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        todo!()
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
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        match self.a.get(access_path)? {
            None => self.b.get(access_path),
            Some(d) => Ok(Some(d)),
        }
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        todo!()
    }

    fn is_genesis(&self) -> bool {
        false
    }
}

#[derive(Clone)]
pub struct RemoteStateAsyncView {
    state_client: StateApiClient,
    state_root: HashValue,
}

impl RemoteStateAsyncView {
    pub async fn from_url(rpc_url: &str, block_number: Option<u64>) -> Result<Self> {
        let rpc_channel: RpcChannel = jsonrpc_client_transports::transports::http::connect(rpc_url)
            .await
            .map_err(|e| anyhow!(format!("{}", e)))?;
        let chain_client: starcoin_rpc_api::chain::ChainApiClient = rpc_channel.clone().into();
        let state_root = match block_number {
            None => {
                let chain_info = chain_client
                    .info()
                    .await
                    .map_err(|e| anyhow!(format!("{}", e)))?;
                chain_info.head.state_root
            }
            Some(n) => {
                let b: Option<BlockView> = chain_client
                    .get_block_by_number(n, None)
                    .await
                    .map_err(|e| anyhow!(format!("{}", e)))?;
                let b = b.ok_or_else(|| anyhow::anyhow!("cannot found block of height {}", n))?;
                b.header.state_root
            }
        };
        let state_client: starcoin_rpc_api::state::StateApiClient = rpc_channel.clone().into();
        Ok(Self {
            state_client,
            state_root,
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
}

#[derive(Clone)]
pub struct RemoteStateView {
    svc: RemoteStateAsyncView,
    rt: Arc<Runtime>,
}

impl RemoteStateView {
    pub fn from_url(rpc_url: &str, block_number: Option<u64>) -> Result<Self> {
        let mut rt = tokio::runtime::Builder::new()
            .thread_name("remote-state-worker")
            .threaded_scheduler()
            .enable_all()
            .build()?;

        let v =
            rt.block_on(async { RemoteStateAsyncView::from_url(rpc_url, block_number).await })?;

        Ok(Self {
            svc: v,
            rt: Arc::new(rt),
        })
    }

    pub fn get_modules(
        &self,
        addr: AccountAddress,
    ) -> VMResult<Option<BTreeMap<Identifier, Vec<u8>>>> {
        let handle = self.rt.handle().clone();
        handle.block_on(self.svc.get_modules_async(addr))
    }
}

impl ModuleResolver for RemoteStateView {
    type Error = VMError;

    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        let handle = self.rt.handle().clone();
        handle.block_on(self.svc.get_module_async(module_id))
    }
}

impl ResourceResolver for RemoteStateView {
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

impl StateView for RemoteStateView {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        match &access_path.path {
            DataPath::Code(m) => Ok(self
                .get_module(&ModuleId::new(access_path.address, m.clone()))
                .map_err(|err| err.into_vm_status())?),
            DataPath::Resource(s) => Ok(self
                .get_resource(&access_path.address, s)
                .map_err(|err| err.finish(Location::Undefined).into_vm_status())?),
        }
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
