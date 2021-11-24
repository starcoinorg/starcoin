// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::account_address::AccountAddress;
use move_core_types::vm_status::sub_status::NFE_BCS_TO_ADDRESS_FAILURE;
use move_vm_runtime::native_functions::NativeContext;
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::{native_gas, NativeResult};
use move_vm_types::pop_arg;
use move_vm_types::values::Value;
use smallvec::smallvec;
use starcoin_vm_types::gas_schedule::NativeCostIndex;
use std::collections::VecDeque;
use std::convert::TryFrom;

/// Rust implementation of Move's `public fun from_public_key_vec(pub_key_vec: vector<u8>): address;`
pub fn native_to_address(
    context: &mut NativeContext,
    mut _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let key_bytes = pop_arg!(args, Vec<u8>);
    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::BCS_TO_ADDRESS as u8,
        key_bytes.len(),
    );
    if key_bytes.len() != AccountAddress::LENGTH {
        return Ok(NativeResult::err(cost, NFE_BCS_TO_ADDRESS_FAILURE));
    }

    let address = match AccountAddress::try_from(&key_bytes[..AccountAddress::LENGTH]) {
        Ok(addr) => addr,
        Err(_) => return Ok(NativeResult::err(cost, NFE_BCS_TO_ADDRESS_FAILURE)),
    };
    let return_values = smallvec![Value::address(address)];
    Ok(NativeResult::ok(cost, return_values))
}
