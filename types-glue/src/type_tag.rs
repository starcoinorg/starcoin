// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_vm2_types::language_storage::TypeTag as TypeTagVM2;
use starcoin_vm_types::language_storage::TypeTag as TypeTagVM1;
use std::str::FromStr;

pub fn vm1_to_vm2(tag: TypeTagVM1) -> TypeTagVM2 {
    TypeTagVM2::from_str(&tag.to_canonical_string()).expect("invalid type tag")
}

pub fn vm2_to_vm1(tag: TypeTagVM2) -> TypeTagVM1 {
    TypeTagVM1::from_str(&tag.to_canonical_string()).expect("invalid type tag")
}
