use move_core_types::gas_algebra::NumBytes;
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    values::{Reference, Struct, StructRef, VMValueCast, Value},
};
use smallvec::{smallvec, SmallVec};
use starcoin_gas_schedule::gas_params::natives::starcoin_framework_legacy::*;
use starcoin_native_interface::{
    safely_pop_arg, RawSafeNative, SafeNativeBuilder, SafeNativeContext, SafeNativeError,
    SafeNativeResult,
};
use starcoin_types::error;
use starcoin_uint::U256;
use std::{collections::VecDeque, convert::TryFrom};

pub mod abort_codes {
    pub const NFE_ARITHMETIC_ERROR: u64 = 0x01_0001;
}

macro_rules! impl_native {
    ($func_name: ident, $op: ident, $gas_const_var: ident) => {
        pub fn $func_name(
            context: &mut SafeNativeContext,
            ty_args: Vec<Type>,
            mut arguments: VecDeque<Value>,
        ) -> SafeNativeResult<SmallVec<[Value; 1]>> {
            debug_assert!(ty_args.is_empty());
            debug_assert!(arguments.len() == 2);

            let b = {
                let b = safely_pop_arg!(arguments, StructRef);
                let field_ref: Reference = b.borrow_field(0)?.cast()?;
                let field: Vec<u64> = field_ref.read_ref()?.cast()?;
                field
            };

            let a_ref = safely_pop_arg!(arguments, StructRef);
            let a = {
                let field_ref: Reference = a_ref.borrow_field(0)?.cast()?;
                let field_value = field_ref.read_ref()?;
                let field: Vec<u64> = field_value.cast()?;
                field
            };
            let a = U256(<[u64; 4]>::try_from(a).unwrap());
            let b = U256(<[u64; 4]>::try_from(b).unwrap());

            let res = match a.$op(b) {
                None => {
                    return Err(SafeNativeError::Abort {
                        abort_code: error::invalid_state(abort_codes::NFE_ARITHMETIC_ERROR),
                    })
                }
                // None => return Err(PartialVMError::new(StatusCode::ARITHMETIC_ERROR)),
                Some(r) => r,
            }
            .0
            .to_vec();

            {
                let field_ref: Reference = a_ref.borrow_field(0)?.cast()?;
                field_ref.write_ref(Value::vector_u64(res))?;
            }
            // let cost = gas_params.base;
            context.charge($gas_const_var)?;
            Ok(smallvec![])
        }
    };
}

impl_native!(native_u256_add, checked_add, U256_ADD_BASE);
impl_native!(native_u256_sub, checked_sub, U256_SUB_BASE);
impl_native!(native_u256_mul, checked_mul, U256_MUL_BASE);
impl_native!(native_u256_div, checked_div, U256_DIV_BASE);
impl_native!(native_u256_rem, checked_rem, U256_REM_BASE);
impl_native!(native_u256_pow, checked_pow, U256_POW_BASE);

/***************************************************************************************************
 * native fun native_u256_pow
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
pub fn native_u256_from_bytes(
    context: &mut SafeNativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 2);
    let big_endian = safely_pop_arg!(arguments, bool);
    let bytes: Vec<u8> = {
        let byte_ref = safely_pop_arg!(arguments, Reference);
        byte_ref.read_ref()?.cast()?
    };

    debug_assert!(bytes.len() <= 32);
    let ret = if big_endian {
        U256::from_big_endian(&bytes)
    } else {
        U256::from_little_endian(&bytes)
    };

    let ret = Value::struct_(Struct::pack(vec![Value::vector_u64(ret.0)]));
    // let cost = gas_params.base + gas_params.per_byte * NumBytes::new(ret.legacy_size().into());
    context.charge(U256_FROM_BYTES_PER_BYTE * NumBytes::new(ret.legacy_size().into()))?;
    Ok(smallvec![ret])
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/

pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    let natives = [
        ("native_add", native_u256_add as RawSafeNative),
        ("native_sub", native_u256_sub as RawSafeNative),
        ("native_mul", native_u256_mul as RawSafeNative),
        ("native_div", native_u256_div as RawSafeNative),
        ("native_rem", native_u256_rem as RawSafeNative),
        ("native_pow", native_u256_pow as RawSafeNative),
        ("from_bytes", native_u256_from_bytes as RawSafeNative),
    ];
    builder.make_named_natives(natives)
}
