// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_table_extension::TableResolver;
use move_vm_types::resolver::MoveResolver;

pub trait MoveResolverExt: MoveResolver + TableResolver {}

impl<T: MoveResolver + TableResolver + ?Sized> MoveResolverExt for T {}
