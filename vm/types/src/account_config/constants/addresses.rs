// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;

pub use move_core_types::language_storage::CORE_CODE_ADDRESS;

pub fn association_address() -> AccountAddress {
    libra_types::account_config::constants::addresses::libra_root_address()
}
pub fn core_code_address() -> AccountAddress {
    CORE_CODE_ADDRESS
}

pub fn genesis_address() -> AccountAddress {
    CORE_CODE_ADDRESS
}

pub fn config_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xF1A95").expect("failed to get address")
}
