// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_vm_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
};

//TODO should remove this trait?
pub trait AccessPathCache {
    fn get_module_path(&mut self, module_id: ModuleId) -> AccessPath;
    fn get_resource_path(&mut self, address: AccountAddress, struct_tag: StructTag) -> AccessPath;
}

impl AccessPathCache for () {
    fn get_module_path(&mut self, module_id: ModuleId) -> AccessPath {
        AccessPath::from(&module_id)
    }

    fn get_resource_path(&mut self, address: AccountAddress, struct_tag: StructTag) -> AccessPath {
        AccessPath::resource_access_path(address, struct_tag)
    }
}
