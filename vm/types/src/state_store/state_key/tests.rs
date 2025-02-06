// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::ObjectGroupResource;
use crate::{account_config::AccountResource, state_store::state_key::StateKey};
use move_core_types::move_resource::MoveStructType;
use move_core_types::{account_address::AccountAddress, ident_str};
use starcoin_crypto::hash::CryptoHash;

fn assert_crypto_hash(key: &StateKey, expected_hash: &str) {
    let expected_hash = expected_hash.parse().unwrap();
    assert_eq!(CryptoHash::hash(key), expected_hash);
}

#[test]
fn test_resource_hash() {
    assert_crypto_hash(
        &StateKey::resource_typed::<AccountResource>(&AccountAddress::TWO).unwrap(),
        "fdec56915926115cd094939bf5ef500157dd63a56e1a0e7521600adacdc50b90",
    );
}

#[test]
fn test_resource_group_hash() {
    assert_crypto_hash(
        &StateKey::resource_group(&AccountAddress::TWO, &ObjectGroupResource::struct_tag()),
        "235dbcd87b398a707c229dbe02d3a42b118128a38ac266ff2d77c729c8c24d42",
    );
}

#[test]
fn test_module_hash() {
    assert_crypto_hash(
        &StateKey::module(&AccountAddress::TWO, ident_str!("mymodule")),
        "2b1fbcded1092bfbd49306ed6f39766c6235b3c52d412965668b600bc874e119",
    );
}

#[test]
fn test_table_item_hash() {
    assert_crypto_hash(
        &StateKey::table_item(&"0x1002".parse().unwrap(), &[7, 2, 3]),
        "0x9ad0a641cf1fd5276bcc6d830502876acc3dcf5dc6e46371381930e6fb04a59c",
    );
}

#[test]
fn test_raw_hash() {
    assert_crypto_hash(
        &StateKey::raw(&[1, 2, 3]),
        "0xc0c9003878aae2e718af90a60bbc5dd3fbfc47c6351e0a67e875977e97ee0443",
    )
}

#[test]
fn test_debug() {
    // code
    let key = StateKey::module(&AccountAddress::ONE, ident_str!("account"));
    assert_eq!(
        &format!("{:?}", key),
        "StateKey::AccessPath { address: 0x1, path: \"Code(0x00000000000000000000000000000001::account)\" }",
    );

    // resource
    let key = StateKey::resource_typed::<AccountResource>(&AccountAddress::FOUR).unwrap();
    assert_eq!(
        &format!("{:?}", key),
        "StateKey::AccessPath { address: 0x4, path: \"Resource(0x00000000000000000000000000000001::account::Account)\" }",
    );

    // resource group
    let key = StateKey::resource_group(&AccountAddress::THREE, &ObjectGroupResource::struct_tag());
    assert_eq!(
        &format!("{:?}", key),
        "StateKey::AccessPath { address: 0x3, path: \"ResourceGroup(0x00000000000000000000000000000001::object::ObjectGroup)\" }",
    );

    // table item
    let key = StateKey::table_item(&"0x123".parse().unwrap(), &[1]);
    assert_eq!(
        &format!("{:?}", key),
        "StateKey::TableItem { handle: 00000000000000000000000000000123, key: 01 }"
    );

    // raw
    let key = StateKey::raw(&[1, 2, 3]);
    assert_eq!(&format!("{:?}", key), "StateKey::Raw(010203)",);
}
