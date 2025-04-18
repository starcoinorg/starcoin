// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_address, struct_tag};
use starcoin_vm2_vm_types as VM2;
use starcoin_vm_types as VM1;

pub fn vm1_to_vm2(path: VM1::access_path::AccessPath) -> VM2::state_store::state_key::StateKey {
    VM2::state_store::state_key::StateKey::resource(
        &account_address::vm1_to_vm2(path.address),
        &struct_tag::vm1_to_vm2(
            path.path
                .as_struct_tag()
                .expect("failed to read struct tag")
                .clone(),
        ),
    )
    .expect("Failed to get access path")
}
