// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_set;
use starcoin_types::state_set::AccountStateSet as AccountStateSetVM1;
use starcoin_vm2_types::state_set::AccountStateSet as AccountStateSetVM2;

pub fn vm1_to_vm2(obj: AccountStateSetVM1) -> AccountStateSetVM2 {
    AccountStateSetVM2::new(
        obj.into_iter()
            .map(|opt_set| opt_set.clone().map(state_set::vm1_to_vm2))
            .collect(),
    )
}

pub fn vm2_to_vm1(obj: AccountStateSetVM2) -> AccountStateSetVM1 {
    AccountStateSetVM1::new(
        obj.into_iter()
            .map(|opt_set| opt_set.clone().map(state_set::vm2_to_vm1))
            .collect(),
    )
}
