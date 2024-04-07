use anyhow::Error;
use move_core_types::vm_status::StatusCode;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
    resolver::{ModuleResolver, ResourceResolver},
};
use move_table_extension::{TableHandle, TableResolver};
use std::ops::Deref;

use crate::create_access_path;
use starcoin_vm_types::errors::{Location, PartialVMError, PartialVMResult};
use starcoin_vm_types::{
    access_path::AccessPath,
    account_config::{genesis_address, ModuleUpgradeStrategy},
    errors::{VMError, VMResult},
    move_resource::MoveResource,
    state_store::state_key::StateKey,
    state_view::StateView,
};

pub const FORCE_UPGRADE_BLOCK_NUMBER: u64 = 17500000;

// Adapter to convert a `StateView` into a `RemoteCache`.
pub struct RemoteStorageForceUpgrade<'a, S>(&'a S);

impl<'a, S: StateView> RemoteStorageForceUpgrade<'a, S> {
    pub fn new(state_store: &'a S) -> Self {
        Self(state_store)
    }

    pub fn get(&self, access_path: &AccessPath) -> PartialVMResult<Option<Vec<u8>>> {
        let strategy_path = AccessPath::resource_access_path(
            genesis_address(),
            ModuleUpgradeStrategy::struct_tag(),
        );

        if *access_path == strategy_path {
            return Ok(Some(vec![100]));
        };

        self.0
            .get_state_value(&StateKey::AccessPath(access_path.clone()))
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR))
    }
}

impl<'a, S: StateView> ModuleResolver for RemoteStorageForceUpgrade<'a, S> {
    type Error = VMError;
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        // REVIEW: cache this?
        let ap = AccessPath::from(module_id);
        self.get(&ap).map_err(|e| e.finish(Location::Undefined))
    }
}
impl<'a, S: StateView> ResourceResolver for RemoteStorageForceUpgrade<'a, S> {
    type Error = VMError;
    fn get_resource(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
    ) -> VMResult<Option<Vec<u8>>> {
        let ap = create_access_path(*address, struct_tag.clone());
        self.get(&ap).map_err(|e| e.finish(Location::Undefined))
    }
}

impl<'a, S> Deref for RemoteStorageForceUpgrade<'a, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, S: StateView> TableResolver for RemoteStorageForceUpgrade<'a, S> {
    fn resolve_table_entry(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, Error> {
        self.0
            .get_state_value(&StateKey::table_item((*handle).into(), key.to_vec()))
    }
}

pub trait AsForceUpgradeResolver<S> {
    fn as_force_upgrade_resolver(&self) -> RemoteStorageForceUpgrade<S>;
}

impl<S: StateView> AsForceUpgradeResolver<S> for S {
    fn as_force_upgrade_resolver(&self) -> RemoteStorageForceUpgrade<S> {
        RemoteStorageForceUpgrade::new(self)
    }
}
