// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::resolver::MoveResolver;
use move_table_extension::TableResolver;
use std::fmt::Debug;

pub trait MoveResolverExt: MoveResolver<Err = Self::ExtError> + TableResolver {
    type ExtError: Debug;
}

impl<E: Debug, T: MoveResolver<Err = E> + TableResolver + ?Sized> MoveResolverExt for T {
    type ExtError = E;
}
