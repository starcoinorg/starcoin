// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_table_extension::TableResolver;
use move_vm_types::resolver::MoveResolver;
use std::fmt::Debug;

pub trait MoveResolverExt: MoveResolver + TableResolver {
    type ExtError: Debug;
}

impl<E: Debug, T: MoveResolver + TableResolver + ?Sized> MoveResolverExt for T {
    type ExtError = E;
}
