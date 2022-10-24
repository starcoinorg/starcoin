// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::account_address::AccountAddress;
use move_core_types::vm_status::sub_status::NFE_BCS_TO_ADDRESS_FAILURE;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::{native_gas, NativeResult};
use move_vm_types::pop_arg;
use move_vm_types::values::Value;
use smallvec::smallvec;
use starcoin_vm_types::gas_schedule::NativeCostIndex;
use std::collections::VecDeque;
use std::convert::TryFrom;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};

/***************************************************************************************************
 * native fun native_to_address
 *
 *   gas cost: base_cost + unit_cost * data_length
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct AddressGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}


/// Rust implementation of Move's `public fun from_public_key_vec(pub_key_vec: vector<u8>): address;`
pub fn native_to_address(
    gas_params: &AddressGasParameters,
    _context: &mut NativeContext,
    mut _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let key = pop_arg!(args, Vec<u8>);
    let cost = gas_params.base + gas_params.per_byte * NumBytes::new(key.len() as u64);
    if key.len() != AccountAddress::LENGTH {
        return Ok(NativeResult::err(cost, NFE_BCS_TO_ADDRESS_FAILURE));
    }

    let address = match AccountAddress::try_from(&key[..AccountAddress::LENGTH]) {
        Ok(addr) => addr,
        Err(_) => return Ok(NativeResult::err(cost, NFE_BCS_TO_ADDRESS_FAILURE)),
    };
    let return_values = smallvec![Value::address(address)];
    Ok(NativeResult::ok(cost, return_values))
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub address: AddressGasParameters,
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "address",
            make_native_from_func(gas_params.address, native_to_address),
        ),
    ];

    crate::natives::helpers::make_module_natives(natives)
}