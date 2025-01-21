// Copyright © Starcoin Foundation
// SPDX-License-Identifier: Apache-2.0

use move_core_types::language_storage::{StructTag, CORE_CODE_ADDRESS};
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};

#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

pub fn primary_store(address: &AccountAddress, meta_address: &AccountAddress) -> AccountAddress {
    let mut bytes = address.to_vec();
    // bytes.append(&mut AccountAddress::TEN.to_vec());
    bytes.append(&mut meta_address.to_vec());
    bytes.push(0xFC);
    let hash_vec = starcoin_crypto::hash::HashValue::sha3_256_of(&bytes).to_vec();
    let address_bytes: [u8; AccountAddress::LENGTH] = hash_vec[16..]
        .try_into()
        .expect("Slice with incorrect length");
    AccountAddress::from_bytes(address_bytes).unwrap()
}
/// The balance resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct FungibleStoreResource {
    metadata: AccountAddress,
    balance: u64,
    frozen: bool,
}

impl FungibleStoreResource {
    pub fn new(metadata: AccountAddress, balance: u64, frozen: bool) -> Self {
        Self {
            metadata,
            balance,
            frozen,
        }
    }

    pub fn metadata(&self) -> AccountAddress {
        self.metadata
    }

    pub fn balance(&self) -> u64 {
        self.balance
    }

    pub fn frozen(&self) -> bool {
        self.frozen
    }

    // TODO/XXX: remove this once the MoveResource trait allows type arguments to `struct_tag`.
    pub fn struct_tag_for_resource() -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            name: Self::struct_identifier(),
            module: Self::module_identifier(),
            type_args: vec![],
        }
    }

    // TODO: remove this once the MoveResource trait allows type arguments to `resource_path`.
    // pub fn access_path_for(token_type_tag: StructTag) -> DataPath {
    //     AccessPath::resource_data_path(Self::struct_tag_for_token(token_type_tag))
    // }
}

impl MoveStructType for FungibleStoreResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("fungible_asset");
    const STRUCT_NAME: &'static IdentStr = ident_str!("FungibleStore");
}

impl MoveResource for FungibleStoreResource {}

#[test]
fn test_compare_primary_store_address() -> anyhow::Result<()> {
    let primary_store_addr = primary_store(
        &AccountAddress::from_hex_literal("0xd0c5a06ae6100ce115cad1600fe59e96")?,
        &AccountAddress::ONE,
    );
    assert_eq!(
        primary_store_addr,
        AccountAddress::from_hex_literal("0x786d516a2228196dff48bf39a4b085f0")?,
        "Not expect address"
    );
    Ok(())
}
