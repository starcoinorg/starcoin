// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use executor::account::Account;
use starcoin_vm_types::account_config::genesis_address;
use std::collections::BTreeMap;

// These are special-cased since they are generated in genesis, and therefore we don't want
// their account states to be generated.
pub const ASSOCIATION_NAME: &str = "association";
pub const GENESIS_NAME: &str = "genesis";

pub fn make_genesis_accounts() -> BTreeMap<String, Account> {
    let mut m = BTreeMap::new();
    m.insert(ASSOCIATION_NAME.to_string(), Account::new_association());
    m.insert(
        GENESIS_NAME.to_string(),
        Account::new_genesis_account(genesis_address()),
    );
    m
}
