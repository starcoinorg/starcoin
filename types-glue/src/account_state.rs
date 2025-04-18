// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_types::account_state::AccountState as AccountStateVM1;
use starcoin_vm2_types::account_state::AccountState as AccountStateVM2;

pub fn vm1_to_vm2(state: AccountStateVM1) -> AccountStateVM2 {
    AccountStateVM2::new(state.code_root(), state.resource_root(), None)
}

pub fn vm2_to_vm1(state: AccountStateVM2) -> AccountStateVM1 {
    AccountStateVM1::new(state.code_root(), state.resource_root())
}
