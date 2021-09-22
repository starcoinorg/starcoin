use crate::dependencies::get_uses;
use anyhow::{anyhow, Result};
use jsonrpc_client_transports::RpcChannel;
use move_vm_runtime::data_cache::MoveStorage;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::ChainApiClient;
use starcoin_rpc_api::state::StateApiClient;
use starcoin_rpc_api::types::{BlockView, StateWithProofView};
use starcoin_types::access_path::{AccessPath, DataPath};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag};
use starcoin_types::vm_error::StatusCode;
use starcoin_vm_types::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use vm::CompiledModule;

pub struct MergedRemoteCache<A: MoveStorage, B: MoveStorage> {
    pub a: A,
    pub b: B,
}

impl<A, B> MoveStorage for MergedRemoteCache<A, B>
where
    A: MoveStorage,
    B: MoveStorage,
{
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        match self.a.get_module(module_id)? {
            Some(d) => Ok(Some(d)),
            None => self.b.get_module(module_id),
        }
    }

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        match self.a.get_resource(address, tag)? {
            Some(d) => Ok(Some(d)),
            None => self.b.get_resource(address, tag),
        }
    }
}

#[derive(Clone)]
pub struct RemoteStateAsyncView {
    state_client: StateApiClient,
    chain_client: ChainApiClient,
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
            chain_client,
            state_root,
        })
    }

    pub async fn resolve_deps_async(
        &self,
        source_files: &[String],
    ) -> Result<Vec<(ModuleId, Vec<u8>)>> {
        let uses = get_uses(source_files)?;
        let mut found_modules = vec![];
        for (addr, name) in uses {
            let module_id = ModuleId::new(
                AccountAddress::new(addr.into_bytes()),
                Identifier::new(name).unwrap(),
            );
            let compiled_module = self
                .get_module_async(&module_id)
                .await
                .map_err(|e| e.into_vm_status())?;
            if let Some(bytes) = compiled_module {
                found_modules.push((module_id, bytes));
            }
        }
        Ok(found_modules)
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

impl MoveStorage for RemoteStateView {
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        let handle = self.rt.handle().clone();
        handle.block_on(self.svc.get_module_async(module_id))
    }

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        let handle = self.rt.handle().clone();
        handle.block_on(self.svc.get_resource_async(address, tag))
    }
}

pub fn resolve_deps(
    state: &dyn MoveStorage,
    source_files: &[String],
) -> Result<Vec<CompiledModule>> {
    let uses = get_uses(source_files)?;
    let mut found_modules = vec![];
    for (addr, name) in uses {
        let module_id = ModuleId::new(
            AccountAddress::new(addr.into_bytes()),
            Identifier::new(name).unwrap(),
        );
        let compiled_module = state
            .get_module(&module_id)
            .map_err(|e| e.into_vm_status())?;
        if let Some(bytes) = compiled_module {
            found_modules.push(CompiledModule::deserialize(&bytes)?);
        }
    }
    Ok(found_modules)
}
