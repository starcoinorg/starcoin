use move_binary_format::errors::PartialVMResult;
use move_vm_runtime::native_functions::NativeContext;
use move_vm_types::{
    gas_schedule::NativeCostIndex,
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeResult},
    pop_arg,
    values::Value,
};
use smallvec::smallvec;
use std::collections::VecDeque;
use tiny_keccak::Hasher;

pub fn native_keccak_256(
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let hash_arg = pop_arg!(arguments, Vec<u8>);

    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::KECCAK_256,
        hash_arg.len(),
    );
    let output = {
        let mut output = [0u8; 32];
        let mut keccak = tiny_keccak::Keccak::v256();
        keccak.update(hash_arg.as_slice());
        keccak.finalize(&mut output);
        output.to_vec()
    };

    Ok(NativeResult::ok(cost, smallvec![Value::vector_u8(output)]))
}
