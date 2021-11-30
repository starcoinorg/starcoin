use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::gas_schedule::GasAlgebra;
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::NativeContext;
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::{native_gas, NativeResult};
use move_vm_types::pop_arg;
use move_vm_types::values::{Reference, Struct, StructRef, VMValueCast, Value};
use smallvec::smallvec;
use starcoin_uint::U256;
use starcoin_vm_types::gas_schedule::NativeCostIndex::*;
use std::collections::VecDeque;
use std::convert::TryFrom;

macro_rules! impl_native {
    ($func:ident,$cost_index: ident, $op:ident) => {
        pub fn $func(
            context: &mut NativeContext,
            _ty_args: Vec<Type>,
            mut arguments: VecDeque<Value>,
        ) -> PartialVMResult<NativeResult> {
            debug_assert!(_ty_args.is_empty());
            debug_assert!(arguments.len() == 2);
            let b = {
                let b = pop_arg!(arguments, StructRef);
                let field_ref: Reference = b.borrow_field(0)?.cast()?;
                let field: Vec<u64> = field_ref.read_ref()?.cast()?;
                field
            };

            let a_ref = pop_arg!(arguments, StructRef);

            let a = {
                let field_ref: Reference = a_ref.borrow_field(0)?.cast()?;
                let field_value = field_ref.read_ref()?;
                let field: Vec<u64> = field_value.cast()?;
                field
            };
            let a = U256(<[u64; 4]>::try_from(a).unwrap());
            let b = U256(<[u64; 4]>::try_from(b).unwrap());

            let res = match a.$op(b) {
                None => return Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR)),
                Some(r) => r,
            }
            .0
            .to_vec();

            {
                let field_ref: Reference = a_ref.borrow_field(0)?.cast()?;
                field_ref.write_ref(Value::vector_u64(res))?;
            }
            let cost = native_gas(context.cost_table(), $cost_index as u8, 1);
            Ok(NativeResult::ok(cost, smallvec![]))
        }
    };
}

impl_native!(native_u256_add, U256_ADD, checked_add);
impl_native!(native_u256_sub, U256_SUB, checked_sub);
impl_native!(native_u256_mul, U256_MUL, checked_mul);
impl_native!(native_u256_div, U256_DIV, checked_div);
impl_native!(native_u256_rem, U256_REM, checked_rem);
impl_native!(native_u256_pow, U256_POW, checked_pow);

pub fn native_u256_from_bytes(
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 2);
    let big_endian = pop_arg!(arguments, bool);
    let bytes: Vec<u8> = {
        let byte_ref = pop_arg!(arguments, Reference);
        byte_ref.read_ref()?.cast()?
    };

    debug_assert!(bytes.len() <= 32);
    let ret = if big_endian {
        U256::from_big_endian(&bytes)
    } else {
        U256::from_little_endian(&bytes)
    };

    let ret = Value::struct_(Struct::pack(vec![Value::vector_u64(ret.0)]));
    let cost = native_gas(
        context.cost_table(),
        U256_FROM_BYTES as u8,
        ret.size().get() as usize,
    );
    Ok(NativeResult::ok(cost, smallvec![ret]))
}
