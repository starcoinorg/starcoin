// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::type_tag;
use starcoin_vm2_vm_types::state_store::table::TableInfo as TableInfoVM2;
use starcoin_vm_types::state_store::table::TableInfo as TableInfoVM1;

pub fn vm1_to_vm2(info: TableInfoVM1) -> TableInfoVM2 {
    TableInfoVM2::new(
        type_tag::vm1_to_vm2(info.key_type),
        type_tag::vm1_to_vm2(info.value_type),
    )
}

pub fn vm2_to_vm1(info: TableInfoVM2) -> TableInfoVM1 {
    TableInfoVM1::new(
        type_tag::vm2_to_vm1(info.key_type),
        type_tag::vm2_to_vm1(info.value_type),
    )
}
