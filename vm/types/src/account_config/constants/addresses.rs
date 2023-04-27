// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
pub use move_core_types::language_storage::CORE_CODE_ADDRESS;
use once_cell::sync::Lazy;

pub fn association_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xA550C18")
        .expect("Parsing valid hex literal should always succeed")
}
pub fn core_code_address() -> AccountAddress {
    CORE_CODE_ADDRESS
}

pub fn genesis_address() -> AccountAddress {
    CORE_CODE_ADDRESS
}

pub const TABLE_ADDRESS_LIST_LEN: usize = 4;
pub const TABLE_ADDRESS_LIST: [&str; TABLE_ADDRESS_LIST_LEN] =
    ["0xA550C68", "0xA550C69", "0xA550C6A", "0xA550C6B"];

pub static TABLE_HANDLE_ADDRESS_LIST: Lazy<Vec<AccountAddress>> = Lazy::new(|| {
    let mut arr = vec![];
    for str in TABLE_ADDRESS_LIST {
        arr.push(
            AccountAddress::from_hex_literal(str)
                .expect("Parsing valid hex literal should always succeed"),
        );
    }
    arr
});
