use crate::util::make_native_from_func;
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::NativeResult;
use move_vm_types::pop_arg;
use move_vm_types::values::{Reference, Struct, StructRef, VMValueCast, Value};
use smallvec::smallvec;
use starcoin_uint::U256;
use std::collections::VecDeque;
use std::convert::TryFrom;

macro_rules! impl_native {
    ($func:ident,$params: ident, $op:ident) => {
        pub fn $func(
            gas_params: &$params,
            _context: &mut NativeContext,
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
            let cost = gas_params.base;
            Ok(NativeResult::ok(cost, smallvec![]))
        }
    };
}

impl_native!(native_u256_add, U256AddGasParameters, checked_add);
impl_native!(native_u256_sub, U256SubGasParameters, checked_sub);
impl_native!(native_u256_mul, U256MulGasParameters, checked_mul);
impl_native!(native_u256_div, U256DivGasParameters, checked_div);
impl_native!(native_u256_rem, U256RemGasParameters, checked_rem);
impl_native!(native_u256_pow, U256PowGasParameters, checked_pow);

/***************************************************************************************************
 * native fun native_u256_add
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct U256AddGasParameters {
    pub base: InternalGas,
}

/***************************************************************************************************
 * native fun native_u256_sub
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct U256SubGasParameters {
    pub base: InternalGas,
}

/***************************************************************************************************
 * native fun native_u256_mul
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct U256MulGasParameters {
    pub base: InternalGas,
}

/***************************************************************************************************
 * native fun native_u256_div
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct U256DivGasParameters {
    pub base: InternalGas,
}

/***************************************************************************************************
 * native fun native_u256_rem
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct U256RemGasParameters {
    pub base: InternalGas,
}

/***************************************************************************************************
 * native fun native_u256_pow
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct U256PowGasParameters {
    pub base: InternalGas,
}

/***************************************************************************************************
 * native fun native_u256_pow
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct U256FromBytesGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

pub fn native_u256_from_bytes(
    gas_params: &U256FromBytesGasParameters,
    _context: &mut NativeContext,
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
    let cost = gas_params.base + gas_params.per_byte * NumBytes::new(ret.legacy_size().into());
    Ok(NativeResult::ok(cost, smallvec![ret]))
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub add: U256AddGasParameters,
    pub sub: U256SubGasParameters,
    pub mul: U256MulGasParameters,
    pub div: U256DivGasParameters,
    pub rem: U256RemGasParameters,
    pub pow: U256PowGasParameters,
    pub from_bytes: U256FromBytesGasParameters,
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "u256add",
            make_native_from_func(gas_params.add, native_u256_add),
        ),
        (
            "u256sub",
            make_native_from_func(gas_params.sub, native_u256_sub),
        ),
        (
            "u256mul",
            make_native_from_func(gas_params.mul, native_u256_mul),
        ),
        (
            "u256div",
            make_native_from_func(gas_params.div, native_u256_div),
        ),
        (
            "u256rem",
            make_native_from_func(gas_params.rem, native_u256_rem),
        ),
        (
            "u25pow",
            make_native_from_func(gas_params.pow, native_u256_pow),
        ),
        (
            "u256frombytes",
            make_native_from_func(gas_params.from_bytes, native_u256_from_bytes),
        ),
    ];

    crate::helpers::make_module_natives(natives)
}
