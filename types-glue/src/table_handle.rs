// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address;
use starcoin_vm2_vm_types::state_store::table::TableHandle as TableHandleVM2;
use starcoin_vm_types::state_store::table::TableHandle as TableHandleVM1;

pub fn vm1_to_vm2(table_handle: TableHandleVM1) -> TableHandleVM2 {
    TableHandleVM2(account_address::vm1_to_vm2(table_handle.0))
}

pub fn vm2_to_vm1(table_handle: TableHandleVM2) -> TableHandleVM1 {
    TableHandleVM1(account_address::vm2_to_vm1(table_handle.0))
}
