// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! MoveVM and Session wrapped, to make sure Starcoin natives and extensions are always installed and
//! taken care of after session finish.
mod resolver;
mod session;
mod vm;

mod warm_vm_cache;

pub(crate) mod write_op_converter;

pub use crate::move_vm_ext::{
    resolver::{AsExecutorView, StarcoinMoveResolver},
    session::{SessionExt, SessionId, SessionOutput},
    vm::MoveVmExt,
};
