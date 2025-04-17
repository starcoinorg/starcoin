// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_vm_ext::session::SessionId;
use crate::move_vm_ext::MoveResolverExt;
use crate::natives;
use move_table_extension::NativeTableContext;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_runtime::native_extensions::NativeContextExtensions;
use move_vm_runtime::session::Session;
use starcoin_gas::NativeGasParameters;
use starcoin_vm_types::errors::{PartialVMResult, VMResult};
use std::ops::Deref;

pub struct MoveVmExt {
    inner: MoveVM,
}

impl MoveVmExt {
    pub fn new(native_gas_params: NativeGasParameters) -> VMResult<Self> {
        Ok(Self {
            inner: MoveVM::new(natives::starcoin_natives(native_gas_params))?,
        })
    }

    pub fn new_session<'r, S: MoveResolverExt>(
        &self,
        remote: &'r S,
        session_id: SessionId,
    ) -> Session<'r, '_, S> {
        let mut extensions = NativeContextExtensions::default();
        extensions.add(NativeTableContext::new(*session_id.as_uuid(), remote));
        self.inner.new_session_with_extensions(remote, extensions)
    }

    pub fn update_native_functions(
        &mut self,
        native_gas_params: NativeGasParameters,
    ) -> PartialVMResult<()> {
        let native_functions = natives::starcoin_natives(native_gas_params);
        self.inner.update_native_functions(native_functions)
    }
}

impl Deref for MoveVmExt {
    type Target = MoveVM;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
