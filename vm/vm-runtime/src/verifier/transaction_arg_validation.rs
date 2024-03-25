// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_vm_ext::MoveResolverExt;
use crate::vm_adapter::SessionAdapter;
use move_binary_format::errors::{Location, PartialVMError, VMError, VMResult};
use move_binary_format::file_format::FunctionDefinitionIndex;
use move_binary_format::file_format_common::read_uleb128_as_u64;
use move_core_types::vm_status::StatusCode;
use move_core_types::{
    account_address::AccountAddress, ident_str, identifier::Identifier, language_storage::ModuleId,
};
use move_vm_runtime::session::LoadedFunctionInstantiation;
use move_vm_types::gas::{GasMeter, UnmeteredGasMeter};
use move_vm_types::loaded_data::runtime_types::Type;
use once_cell::sync::Lazy;
use starcoin_vm_types::language_storage::FunctionId;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::io::{Cursor, Read};

type ConstructorMap = Lazy<BTreeMap<String, FunctionId>>;

static ALLOWED_STRUCTS: ConstructorMap = Lazy::new(|| {
    [
        (
            "0x1::string::String",
            FunctionId {
                module: ModuleId::new(AccountAddress::ONE, Identifier::from(ident_str!("string"))),
                function: Identifier::from(ident_str!("utf8")),
            },
        ),
        // not implemented in starcoin-framework
        (
            "0x1::object::Object",
            FunctionId {
                module: ModuleId::new(AccountAddress::ONE, Identifier::from(ident_str!("object"))),
                function: Identifier::from(ident_str!("address_to_object")),
            },
        ),
        // not implemented in starcoin-framework
        (
            "0x1::option::Option",
            FunctionId {
                module: ModuleId::new(AccountAddress::ONE, Identifier::from(ident_str!("option"))),
                function: Identifier::from(ident_str!("from_vec")),
            },
        ),
        (
            "0x1::fixed_point32::FixedPoint32",
            FunctionId {
                module: ModuleId::new(
                    AccountAddress::ONE,
                    Identifier::from(ident_str!("fixed_point32")),
                ),
                function: Identifier::from(ident_str!("create_from_raw_value")),
            },
        ),
        (
            "0x1::fixed_point64::FixedPoint64",
            FunctionId {
                module: ModuleId::new(
                    AccountAddress::ONE,
                    Identifier::from(ident_str!("fixed_point64")),
                ),
                function: Identifier::from(ident_str!("create_from_raw_value")),
            },
        ),
    ]
    .into_iter()
    .map(|(s, validator)| (s.to_string(), validator))
    .collect()
});

fn get_allowed_structs() -> &'static ConstructorMap {
    &ALLOWED_STRUCTS
}

/// Validate and generate args for entry function
/// validation includes:
/// 1. return signature is empty
/// 2. number of signers is same as the number of senders
/// 3. check arg types are allowed after signers
///
pub(crate) fn validate_combine_singer_and_args<S: MoveResolverExt>(
    session: &mut SessionAdapter<S>,
    senders: Vec<AccountAddress>,
    args: &[impl Borrow<[u8]>],
    func: &LoadedFunctionInstantiation,
) -> VMResult<()> {
    SessionAdapter::<S>::check_script_return(func.return_.clone())?;

    let mut signer_param_cnt = 0;
    // find all signer params at the beginning
    for ty in func.parameters.iter() {
        match ty {
            Type::Signer => signer_param_cnt += 1,
            Type::Reference(inner_type) => {
                if matches!(&**inner_type, Type::Signer) {
                    signer_param_cnt += 1;
                }
            }
            _ => (),
        }
    }

    let allowed_structs = get_allowed_structs();
    // Need to keep this here to ensure we return the historic correct error code for replay
    for ty in func.parameters[signer_param_cnt..].iter() {
        let valid = is_valid_txn_arg(
            &session,
            &ty.subst(&func.type_arguments).unwrap(),
            allowed_structs,
        );
        if !valid {
            return Err(
                PartialVMError::new(StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE)
                    .with_message(
                        "Found un-allowed parameter which is not signer-params".to_string(),
                    )
                    .finish(Location::Script),
            );
        }
    }

    if (signer_param_cnt + args.len()) != func.parameters.len() {
        return Err(
            PartialVMError::new(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH)
                .with_message(format!(
                    "signer params {signer_param_cnt}, args {}, func parameters {}",
                    args.len(),
                    func.parameters.len()
                ))
                .finish(Location::Script),
        );
    }

    // If the invoked function expects one or more signers, we need to check that the number of
    // signers actually passed is matching first to maintain backward compatibility before
    // moving on to the validation of non-signer args.
    // the number of txn senders should be the same number of signers
    if signer_param_cnt > 0 && senders.len() != signer_param_cnt {
        return Err(
            PartialVMError::new(StatusCode::NUMBER_OF_SIGNER_ARGUMENTS_MISMATCH)
                .finish(Location::Script),
        );
    }

    // This also validates that the args are valid. If they are structs, they have to be allowed
    // and must be constructed successfully. If construction fails, this would fail with a
    // FAILED_TO_DESERIALIZE_ARGUMENT error.
    let _ = construct_args(
        session,
        &func.parameters[signer_param_cnt..],
        args,
        &func.type_arguments,
        allowed_structs,
        false,
    )?;

    Ok(())
}

// Return whether the argument is valid/allowed and whether it needs construction.
pub(crate) fn is_valid_txn_arg<S: MoveResolverExt>(
    session: &SessionAdapter<S>,
    typ: &Type,
    allowed_structs: &ConstructorMap,
) -> bool {
    use move_vm_types::loaded_data::runtime_types::Type::*;

    match typ {
        Bool | U8 | U16 | U32 | U64 | U128 | U256 | Address => true,
        Vector(inner) => is_valid_txn_arg(session, inner, allowed_structs),
        Struct(idx) | StructInstantiation(idx, _) => session
            .inner
            .get_struct_type(*idx)
            .map(|st| {
                let full_name = format!("{}::{}", st.module.short_str_lossless(), st.name);
                allowed_structs.contains_key(&full_name)
            })
            .unwrap_or_default(),
        Signer | Reference(_) | MutableReference(_) | TyParam(_) => false,
    }
}

// Construct arguments. Walk through the arguments and according to the signature
// construct arguments that require so.
// TODO: This needs a more solid story and a tighter integration with the VM.
pub(crate) fn construct_args<S: MoveResolverExt>(
    session: &mut SessionAdapter<S>,
    types: &[Type],
    args: &[impl Borrow<[u8]>],
    ty_args: &[Type],
    allowed_structs: &ConstructorMap,
    is_view: bool,
) -> VMResult<Vec<Vec<u8>>> {
    // Perhaps in a future we should do proper gas metering here
    let mut gas_meter = UnmeteredGasMeter;
    let mut res_args = vec![];
    if types.len() != args.len() {
        return Err(invalid_signature());
    }
    for (ty, arg) in types.iter().zip(args) {
        let arg = construct_arg(
            session,
            &ty.subst(ty_args).unwrap(),
            allowed_structs,
            arg.borrow().to_vec(),
            &mut gas_meter,
            is_view,
        )?;
        res_args.push(arg);
    }
    Ok(res_args)
}

fn invalid_signature() -> VMError {
    PartialVMError::new(StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE).finish(Location::Script)
}

fn construct_arg<S: MoveResolverExt>(
    session: &mut SessionAdapter<S>,
    ty: &Type,
    allowed_structs: &ConstructorMap,
    arg: Vec<u8>,
    gas_meter: &mut impl GasMeter,
    is_view: bool,
) -> VMResult<Vec<u8>> {
    use move_vm_types::loaded_data::runtime_types::Type::*;
    match ty {
        Bool | U8 | U16 | U32 | U64 | U128 | U256 | Address => Ok(arg),
        Vector(_) | Struct { .. } | StructInstantiation { .. } => {
            let initial_cursor_len = arg.len();
            let mut cursor = Cursor::new(&arg[..]);
            let mut new_arg = vec![];
            let mut max_invocations = 10; // Read from config in the future
            recursively_construct_arg(
                session,
                ty,
                allowed_structs,
                &mut cursor,
                initial_cursor_len,
                gas_meter,
                &mut max_invocations,
                &mut new_arg,
            )?;
            // Check cursor has parsed everything
            // Unfortunately, is_empty is only enabled in nightly, so we check this way.
            if cursor.position() != initial_cursor_len as u64 {
                return Err(
                    PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                        .with_message(String::from(
                            "The serialized arguments to constructor contained extra data",
                        ))
                        .finish(Location::Script),
                );
            }
            Ok(new_arg)
        }
        Signer => {
            if is_view {
                Ok(arg)
            } else {
                Err(invalid_signature())
            }
        }
        Reference(_) | MutableReference(_) | TyParam(_) => Err(invalid_signature()),
    }
}

// A Cursor is used to recursively walk the serialized arg manually and correctly. In effect we
// are parsing the BCS serialized implicit constructor invocation tree, while serializing the
// constructed types into the output parameter arg.
pub(crate) fn recursively_construct_arg<S: MoveResolverExt>(
    session: &mut SessionAdapter<S>,
    ty: &Type,
    allowed_structs: &ConstructorMap,
    cursor: &mut Cursor<&[u8]>,
    initial_cursor_len: usize,
    gas_meter: &mut impl GasMeter,
    max_invocations: &mut u64,
    arg: &mut Vec<u8>,
) -> VMResult<()> {
    use move_vm_types::loaded_data::runtime_types::Type::*;

    match ty {
        Vector(inner) => {
            // get the vector length and iterate over each element
            let mut len = get_len(cursor)?;
            serialize_uleb128(len, arg);
            while len > 0 {
                recursively_construct_arg(
                    session,
                    inner,
                    allowed_structs,
                    cursor,
                    initial_cursor_len,
                    gas_meter,
                    max_invocations,
                    arg,
                )?;
                len -= 1;
            }
        }
        Struct(idx) | StructInstantiation(idx, _) => {
            let st = session
                .inner
                .get_struct_type(*idx)
                .ok_or_else(invalid_signature)?;

            let full_name = format!("{}::{}", st.module.short_str_lossless(), st.name);
            let constructor = allowed_structs
                .get(&full_name)
                .ok_or_else(invalid_signature)?;
            // By appending the BCS to the output parameter we construct the correct BCS format
            // of the argument.
            arg.append(&mut validate_and_construct(
                session,
                ty,
                constructor,
                allowed_structs,
                cursor,
                initial_cursor_len,
                gas_meter,
                max_invocations,
            )?);
        }
        Bool | U8 => read_n_bytes(1, cursor, arg)?,
        U16 => read_n_bytes(2, cursor, arg)?,
        U32 => read_n_bytes(4, cursor, arg)?,
        U64 => read_n_bytes(8, cursor, arg)?,
        U128 => read_n_bytes(16, cursor, arg)?,
        U256 | Address => read_n_bytes(32, cursor, arg)?,
        Signer | Reference(_) | MutableReference(_) | TyParam(_) => return Err(invalid_signature()),
    };
    Ok(())
}

// A move function that constructs a type will return the BCS serialized representation of the
// constructed value. This is the correct data to pass as the argument to a function taking
// said struct as a parameter. In this function we execute the constructor constructing the
// value and returning the BCS serialized representation.
fn validate_and_construct<S: MoveResolverExt>(
    session: &mut SessionAdapter<S>,
    expected_type: &Type,
    constructor: &FunctionId,
    allowed_structs: &ConstructorMap,
    cursor: &mut Cursor<&[u8]>,
    initial_cursor_len: usize,
    gas_meter: &mut impl GasMeter,
    max_invocations: &mut u64,
) -> VMResult<Vec<u8>> {
    if *max_invocations == 0 {
        return Err(
            PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                .finish(Location::Undefined),
        );
    }
    // HACK mitigation of performance attack
    // To maintain compatibility with vector<string> or so on, we need to allow unlimited strings.
    // So we do not count the string constructor against the max_invocations, instead we
    // shortcut the string case to avoid the performance attack.
    if constructor.function.as_str() == "utf8" {
        let constructor_error = || {
            // A slight hack, to prevent additional piping of the feature flag through all
            // function calls. We know the feature is active when more structs then just strings are
            // allowed.
            let are_struct_constructors_enabled = allowed_structs.len() > 1;
            if are_struct_constructors_enabled {
                PartialVMError::new(StatusCode::ABORTED)
                    .with_sub_status(1)
                    .at_code_offset(FunctionDefinitionIndex::new(0), 0)
                    .finish(Location::Module(constructor.module.clone()))
            } else {
                PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                    .finish(Location::Undefined)
            }
        };
        // Short cut for the utf8 constructor, which is a special case.
        let len = get_len(cursor)?;
        if !cursor
            .position()
            .checked_add(len as u64)
            .map(|l| l <= initial_cursor_len as u64)
            .unwrap_or_default()
        {
            // We need to make sure we do not allocate more bytes than
            // needed.
            return Err(
                PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                    .with_message("String argument is too long".to_string())
                    .finish(Location::Undefined),
            );
        }

        let mut arg = vec![];
        read_n_bytes(len, cursor, &mut arg)?;
        std::str::from_utf8(&arg).map_err(|_| constructor_error())?;
        return bcs_ext::to_bytes(&arg).map_err(|_| {
            PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                .finish(Location::Undefined)
        });
    } else {
        *max_invocations -= 1;
    }

    let (module, function, instantiation) = session.inner.load_function_with_type_arg_inference(
        &constructor.module,
        constructor.function.as_ref(),
        expected_type,
    )?;
    let mut args = vec![];
    for param_type in &instantiation.parameters {
        let mut arg = vec![];
        recursively_construct_arg(
            session,
            &param_type.subst(&instantiation.type_arguments).unwrap(),
            allowed_structs,
            cursor,
            initial_cursor_len,
            gas_meter,
            max_invocations,
            &mut arg,
        )?;
        args.push(arg);
    }
    let serialized_result = session.inner.execute_instantiated_function(
        module,
        function,
        instantiation,
        args,
        gas_meter,
    )?;
    let mut ret_vals = serialized_result.return_values;
    // We know ret_vals.len() == 1
    let deserialize_error = PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
        .with_message(String::from("Constructor did not return value"))
        .finish(Location::Undefined);
    Ok(ret_vals.pop().ok_or(deserialize_error)?.0)
}

// String is a vector of bytes, so both string and vector carry a length in the serialized format.
// Length of vectors in BCS uses uleb128 as a compression format.
fn get_len(cursor: &mut Cursor<&[u8]>) -> VMResult<usize> {
    match read_uleb128_as_u64(cursor) {
        Err(_) => Err(
            PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                .finish(Location::Undefined),
        ),
        Ok(len) => Ok(len as usize),
    }
}

fn serialize_uleb128(mut x: usize, dest: &mut Vec<u8>) {
    // TODO perhaps reuse the code from move_binary_format::file_format_common if it's public
    while x >= 128 {
        dest.push((x | 128) as u8);
        x >>= 7;
    }
    dest.push(x as u8);
}

fn read_n_bytes(n: usize, src: &mut Cursor<&[u8]>, dest: &mut Vec<u8>) -> VMResult<()> {
    let deserialization_error = |msg: &str| -> VMError {
        PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
            .with_message(msg.to_string())
            .finish(Location::Undefined)
    };
    let len = dest.len();

    // It is safer to limit the length under some big (but still reasonable
    // number).
    const MAX_NUM_BYTES: usize = 1_000_000;
    if !len
        .checked_add(n)
        .map(|s| s <= MAX_NUM_BYTES)
        .unwrap_or_default()
    {
        return Err(deserialization_error(&format!(
            "Couldn't read bytes: maximum limit of {} bytes exceeded",
            MAX_NUM_BYTES
        )));
    }

    // Ensure we have enough capacity for resizing.
    dest.try_reserve(len + n)
        .map_err(|e| deserialization_error(&format!("Couldn't read bytes: {}", e)))?;
    dest.resize(len + n, 0);
    src.read_exact(&mut dest[len..])
        .map_err(|_| deserialization_error("Couldn't read bytes"))
}
