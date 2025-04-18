// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_address, account_state_set};
use starcoin_types::state_set as VM1;
use starcoin_vm2_types::state_set as VM2;

pub fn vm1_to_vm2(obj: VM1::ChainStateSet) -> VM2::ChainStateSet {
    VM2::ChainStateSet::new(
        obj.into_inner()
            .into_iter()
            .map(|(addr, acc_set)| {
                (
                    account_address::vm1_to_vm2(addr),
                    account_state_set::vm1_to_vm2(acc_set),
                )
            })
            .collect(),
    )
}

pub fn vm2_to_vm1(obj: VM2::ChainStateSet) -> VM1::ChainStateSet {
    VM1::ChainStateSet::new(
        obj.into_inner()
            .into_iter()
            .map(|(addr, acc_set)| {
                (
                    account_address::vm2_to_vm1(addr),
                    account_state_set::vm2_to_vm1(acc_set),
                )
            })
            .collect(),
    )
}
