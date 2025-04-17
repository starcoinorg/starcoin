// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::ops::Deref;

// define_type_convert!(
//     AccountAddress1,
//     starcoin_vm_types::account_address::AccountAddress,
//     AccountAddress2,
//     starcoin_vm2_vm_types::account_address::AccountAddress
// );

#[derive(Clone, Copy)]
pub struct AccountAddress1(pub starcoin_vm_types::account_address::AccountAddress);
impl Deref for AccountAddress1 {
    type Target = starcoin_vm_types::account_address::AccountAddress;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Clone, Copy)]
pub struct AccountAddress2(pub starcoin_vm2_vm_types::account_address::AccountAddress);

impl Deref for AccountAddress2 {
    type Target = starcoin_vm2_vm_types::account_address::AccountAddress;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<AccountAddress1> for AccountAddress2 {
    fn from(src: AccountAddress1) -> Self {
        Self(starcoin_vm2_vm_types::account_address::AccountAddress::new(
            src.0.into_bytes(),
        ))
    }
}
impl From<AccountAddress2> for AccountAddress1 {
    fn from(src: AccountAddress2) -> Self {
        Self(starcoin_vm_types::account_address::AccountAddress::new(
            src.0.into_bytes(),
        ))
    }
}

#[test]
fn test_convert_address() {
    let address_type_1 =
        AccountAddress1(starcoin_vm_types::account_address::AccountAddress::random());
    let address_type_2: AccountAddress2 = address_type_1.clone().into();
    assert_eq!(address_type_2.into_bytes(), address_type_1.into_bytes())
}

#[derive(Clone, Copy)]
pub struct HashValue1(pub starcoin_crypto::HashValue);

impl Deref for HashValue1 {
    type Target = starcoin_crypto::HashValue;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy)]
pub struct HashValue2(pub starcoin_vm2_crypto::HashValue);

impl Deref for HashValue2 {
    type Target = starcoin_vm2_crypto::HashValue;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<HashValue1> for HashValue2 {
    fn from(src: HashValue1) -> Self {
        Self(starcoin_vm2_crypto::HashValue::new(src.0.to_inner()))
    }
}
impl From<HashValue2> for HashValue1 {
    fn from(src: HashValue2) -> Self {
        Self(starcoin_crypto::HashValue::new(src.0.to_inner()))
    }
}

#[test]
fn test_convert_hash_value() {
    let hashvalue1 = HashValue1(starcoin_crypto::HashValue::random());
    let address_type_2: HashValue2 = hashvalue1.clone().into();
    assert_eq!(address_type_2.0.to_hex(), address_type_2.0.to_hex())
}
