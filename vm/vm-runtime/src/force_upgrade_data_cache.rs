use std::ops::Deref;

use anyhow::Error;
use move_core_types::vm_status::StatusCode;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
    resolver::{ModuleResolver, ResourceResolver},
};
use move_table_extension::{TableHandle, TableResolver};
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};

use starcoin_types::account::Account;
use starcoin_vm_types::errors::{Location, PartialVMError, PartialVMResult};
use starcoin_vm_types::{
    access_path::AccessPath,
    account_config::{genesis_address, ModuleUpgradeStrategy},
    errors::{VMError, VMResult},
    genesis_config::ChainId,
    move_resource::MoveResource,
    state_store::state_key::StateKey,
    state_view::StateView,
};

use crate::create_access_path;

pub const FORCE_UPGRADE_BLOCK_NUMBER: u64 = 17500000;

pub fn get_force_upgrade_block_number(chain_id: &ChainId) -> u64 {
    if chain_id.is_dev() || chain_id.is_test() {
        5
    } else if chain_id.is_halley() || chain_id.is_proxima() {
        100
    } else if chain_id.is_barnard() {
        21478200
    } else {
        FORCE_UPGRADE_BLOCK_NUMBER
    }
}

fn create_account(private_hex: &str) -> anyhow::Result<Account> {
    let bytes = hex::decode(private_hex)?;
    let private_key = Ed25519PrivateKey::try_from(&bytes[..])?;
    let public_key = Ed25519PublicKey::from(&private_key);
    Ok(Account::with_keypair(
        private_key.into(),
        public_key.into(),
        None,
    ))
}

pub fn get_force_upgrade_account(chain_id: &ChainId) -> anyhow::Result<Account> {
    if chain_id.is_main() {
        // 0x2dd7136c13ed8051fb20147f373f6120 TODO(BobOng): to fill private key
        create_account("")
    } else if chain_id.is_barnard() || chain_id.is_proxima() || chain_id.is_halley() {
        // 0x0b1d07ae560c26af9bbb8264f4c7ee73
        create_account("6105e78821ace0676faf437fb40dd6892e72f01c09351298106bad2964edb007")
    } else {
        Ok(Account::new_association())
    }
}

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

#[test]
fn test_get_force_upgrade_account() -> anyhow::Result<()> {
    // Main TODO(BobOng): To fixed
    // assert_eq!(get_force_upgrade_account(&ChainId::new(1))?.address(), AccountAddress::from_hex_literal("0x2dd7136c13ed8051fb20147f373f6120"));
    // Barnard 251
    assert_eq!(
        *get_force_upgrade_account(&ChainId::new(251))?.address(),
        AccountAddress::from_hex_literal("0x0b1d07ae560c26af9bbb8264f4c7ee73")?
    );
    // Proxima 252
    assert_eq!(
        get_force_upgrade_account(&ChainId::new(252))?.address(),
        &AccountAddress::from_hex_literal("0x0b1d07ae560c26af9bbb8264f4c7ee73")?
    );
    // Halley 253
    assert_eq!(
        get_force_upgrade_account(&ChainId::new(253))?.address(),
        &AccountAddress::from_hex_literal("0x0b1d07ae560c26af9bbb8264f4c7ee73")?
    );
    // Dev 254
    assert_eq!(
        get_force_upgrade_account(&ChainId::new(254))?.address(),
        &AccountAddress::from_hex_literal("0xA550C18")?
    );
    // Test 255
    assert_eq!(
        get_force_upgrade_account(&ChainId::new(254))?.address(),
        &AccountAddress::from_hex_literal("0xA550C18")?
    );

    Ok(())
}
