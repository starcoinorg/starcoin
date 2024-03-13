// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! MoveVM and Session wrapped, to make sure Starcoin natives and extensions are always installed and
//! taken care of after session finish.
mod resolver;
mod session;
mod vm;
mod vm_adapter;

pub use crate::move_vm_ext::{
    resolver::MoveResolverExt,
    session::SessionId,
    session::SessionOutput,
    vm::MoveVmExt,
    vm_adapter::{PublishModuleBundleOption, SessionAdapter},
};
