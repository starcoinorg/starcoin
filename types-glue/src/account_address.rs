// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_types::account_address as VM1;
use starcoin_vm2_types::account_address as VM2;

pub fn vm1_to_vm2(addr: VM1::AccountAddress) -> VM2::AccountAddress {
    VM2::AccountAddress::new(addr.into_bytes())
}

pub fn vm2_to_vm1(addr: VM2::AccountAddress) -> VM1::AccountAddress {
    VM1::AccountAddress::new(addr.into_bytes())
}
