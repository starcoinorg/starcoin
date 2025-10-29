// SafeNative style implementation
use move_core_types::account_address::AccountAddress;
use move_vm_runtime::native_functions::NativeFunction;
use once_cell::sync::Lazy;
use smallvec::{smallvec, SmallVec};
use starcoin_native_interface::{
    safely_pop_arg, RawSafeNative, SafeNativeBuilder, SafeNativeContext, SafeNativeResult,
};
use starcoin_vm_types::{loaded_data::runtime_types::Type, values::Value};
use std::collections::{HashSet, VecDeque};
use std::sync::Mutex;

static PAYER_ADDRESS_CACHE: Lazy<Mutex<HashSet<AccountAddress>>> =
    Lazy::new(|| Mutex::new(HashSet::new()));

fn native_record_payer_address(
    _context: &mut SafeNativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    let addr = safely_pop_arg!(args, AccountAddress);
    let mut cache = PAYER_ADDRESS_CACHE.lock().unwrap();
    cache.insert(addr);
    Ok(smallvec![])
}

fn native_read_and_clear_payer_address(
    _context: &mut SafeNativeContext,
    _ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(args.is_empty());
    let mut cache = PAYER_ADDRESS_CACHE.lock().unwrap();
    let mut addresses: Vec<AccountAddress> = cache.drain().collect();
    addresses.sort();
    addresses.dedup();
    Ok(smallvec![Value::vector_address(addresses)])
}

pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    let natives: [(&str, RawSafeNative); 2] = [
        ("record_payer_address", native_record_payer_address),
        (
            "read_and_clear_payer_address",
            native_read_and_clear_payer_address,
        ),
    ];
    builder.make_named_natives(natives)
}
