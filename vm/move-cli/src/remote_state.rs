use crate::dependencies::get_uses;
use anyhow::{anyhow, Result};
use starcoin_rpc_api::state::StateApiClient;
use starcoin_types::access_path::{AccessPath, DataPath};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag};
use starcoin_types::vm_error::StatusCode;
use starcoin_vm_types::errors::{Location, PartialVMError, PartialVMResult, VMResult};

#[derive(Clone)]
pub struct RemoteStateView {
    state_client: StateApiClient,
}

impl RemoteStateView {
    pub async fn from_url(rpc_url: &str) -> Result<Self> {
        let state_client: starcoin_rpc_api::state::StateApiClient =
            jsonrpc_client_transports::transports::http::connect(rpc_url)
                .await
                .map_err(|e| anyhow!(format!("{}", e)))?;
        Ok(Self { state_client })
    }

    pub async fn resolve_deps(&self, source_files: &[String]) -> Result<Vec<(ModuleId, Vec<u8>)>> {
        let uses = get_uses(&source_files)?;
        let mut found_modules = vec![];
        for (addr, name) in uses {
            let module_id = ModuleId::new(
                AccountAddress::new(addr.to_u8()),
                Identifier::new(name).unwrap(),
            );
            let compiled_module = self
                .get_module(&module_id)
                .await
                .map_err(|e| e.into_vm_status())?;
            if let Some(bytes) = compiled_module {
                found_modules.push((module_id, bytes));
            }
        }
        Ok(found_modules)
    }

    pub async fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        let ap = AccessPath::new(
            *module_id.address(),
            DataPath::Code(module_id.name().to_owned()),
        );
        self.state_client
            .get(ap)
            .await
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined))
    }

    pub async fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        let ap = AccessPath::new(*address, DataPath::Resource(tag.clone()));
        self.state_client
            .get(ap)
            .await
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR))
    }
}
