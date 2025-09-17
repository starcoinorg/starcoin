// Copyright © Starcoin Foundation
// SPDX-License-Identifier: Apache-2.0
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{loaded_data::runtime_types::Type, values::Value};
use smallvec::smallvec;
use smallvec::SmallVec;
use starcoin_native_interface::{SafeNativeBuilder, SafeNativeContext, SafeNativeResult};
use std::collections::VecDeque;

/***************************************************************************************************
 * native fun read<IntElement>(aggregator: &Aggregator<IntElement>): IntElement;
 **************************************************************************************************/
use std::sync::atomic::AtomicU64;
static ATOMIC_COUNTER: AtomicU64 = AtomicU64::new(1);
fn native_atomic_counter_fetch_add(
    _context: &mut SafeNativeContext,
    _ty_args: Vec<Type>,
    mut _args: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    let value =
        Value::u128(ATOMIC_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst) as u128);
    Ok(smallvec![value])
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    let natives = [("atomic_counter_fetch_add", native_atomic_counter_fetch_add)];

    builder.make_named_natives(natives)
}
