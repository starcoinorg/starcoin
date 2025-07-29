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

pub const TABLE_ADDRESS_LIST_LEN: usize = 32;
pub const TABLE_ADDRESS_LIST: [&str; TABLE_ADDRESS_LIST_LEN] = [
    "0x31", "0x32", "0x33", "0x34", "0x35", "0x36", "0x37", "0x38", "0x39", "0x3a", "0x3b", "0x3c",
    "0x3d", "0x3e", "0x3f", "0x40", "0x41", "0x42", "0x43", "0x44", "0x45", "0x46", "0x47", "0x48",
    "0x49", "0x4a", "0x4b", "0x4c", "0x4d", "0x4e", "0x4f", "0x50",
];

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

#[cfg(test)]
mod tests {
    use crate::account_config::{TABLE_ADDRESS_LIST, TABLE_ADDRESS_LIST_LEN};
    use std::collections::HashSet;

    #[test]
    fn test_table_handle_address_list_unique() {
        let table_address_set: HashSet<String> = TABLE_ADDRESS_LIST
            .iter()
            .map(|str| String::from(*str))
            .collect();
        assert_eq!(table_address_set.len(), TABLE_ADDRESS_LIST_LEN);
    }
}
