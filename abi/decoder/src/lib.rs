// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_binary_format::CompiledModule;
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use starcoin_abi_resolver::ABIResolver;
use starcoin_abi_types::TypeInstantiation;
use starcoin_resource_viewer::module_cache::ModuleCache;
use starcoin_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::{ModuleId, TypeTag};
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::{Module, Package, Script, ScriptFunction, TransactionPayload};
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodedTransactionPayload {
    /// A transaction that executes code.
    Script(DecodedScript),
    /// A transaction that publish or update module code by a package.
    Package(DecodedPackage),
    /// A transaction that executes an existing script function published on-chain.
    ScriptFunction(DecodedScriptFunction),
}
impl From<DecodedScript> for DecodedTransactionPayload {
    fn from(d: DecodedScript) -> Self {
        Self::Script(d)
    }
}
impl From<DecodedPackage> for DecodedTransactionPayload {
    fn from(d: DecodedPackage) -> Self {
        Self::Package(d)
    }
}
impl From<DecodedScriptFunction> for DecodedTransactionPayload {
    fn from(d: DecodedScriptFunction) -> Self {
        Self::ScriptFunction(d)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedScript {
    pub code: Vec<u8>,
    pub ty_args: Vec<TypeTag>,
    pub args: Vec<DecodedMoveValue>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedScriptFunction {
    pub module: ModuleId,
    pub function: Identifier,
    pub ty_args: Vec<TypeTag>,
    pub args: Vec<DecodedMoveValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedPackage {
    ///Package's all Module must at same address.
    pub package_address: AccountAddress,
    pub modules: Vec<Module>,
    pub init_script: Option<DecodedScriptFunction>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct DecodedMoveValue(pub serde_json::Value);
impl From<DecodedMoveValue> for serde_json::Value {
    fn from(v: DecodedMoveValue) -> Self {
        v.0
    }
}

/// Transform AnnotatedMoveValue into DecodedMoveValue.
fn struct_to_json(origin: AnnotatedMoveStruct) -> serde_json::Value {
    let mut map = serde_json::Map::with_capacity(origin.value.len());
    for (field_name, fv) in origin.value.into_iter() {
        map.insert(field_name.to_string(), value_to_json(fv));
    }
    serde_json::Value::Object(map)
}

fn value_to_json(origin: AnnotatedMoveValue) -> serde_json::Value {
    use serde_json::Value;
    match origin {
        AnnotatedMoveValue::U8(v) => Value::Number(v.into()),
        AnnotatedMoveValue::U64(v) => Value::Number(v.into()),
        AnnotatedMoveValue::U128(v) => Value::Number(v.into()),
        AnnotatedMoveValue::Bool(v) => Value::Bool(v),
        AnnotatedMoveValue::Address(v) => Value::String(v.to_string()),
        AnnotatedMoveValue::Vector(v) => Value::Array(v.into_iter().map(value_to_json).collect()),
        // try bytes to string, or else to hex string.
        AnnotatedMoveValue::Bytes(v) => Value::String(format!("0x{}", hex::encode(v.as_slice()))),
        AnnotatedMoveValue::Struct(v) => struct_to_json(v),
    }
}
impl From<AnnotatedMoveStruct> for DecodedMoveValue {
    fn from(origin: AnnotatedMoveStruct) -> Self {
        DecodedMoveValue(struct_to_json(origin))
    }
}
impl From<AnnotatedMoveValue> for DecodedMoveValue {
    fn from(origin: AnnotatedMoveValue) -> Self {
        DecodedMoveValue(value_to_json(origin))
    }
}

/// Decode bcs data to json value through type abi.
pub fn decode_move_value(
    abi: &TypeInstantiation,
    data: &[u8],
) -> Result<DecodedMoveValue, bcs::Error> {
    let json_value = bcs::from_bytes_seed(abi, data)?;
    Ok(DecodedMoveValue(json_value))
}

/// Decode transaction payload to human-readable json value through type abi.  
pub fn decode_txn_payload(
    state: &dyn StateView,
    payload: &TransactionPayload,
) -> Result<DecodedTransactionPayload> {
    match payload {
        TransactionPayload::Script(s) => decode_script(state, s).map(Into::into),
        TransactionPayload::Package(pkg) => decode_package(state, pkg).map(Into::into),
        TransactionPayload::ScriptFunction(sf) => decode_script_function(state, sf).map(Into::into),
    }
}

pub fn decode_script(state: &dyn StateView, s: &Script) -> Result<DecodedScript> {
    let resolver = ABIResolver::new(state);
    let script_abi = resolver.resolve_script(s.code().to_vec())?;
    let arg_abis = {
        let arg_abis = script_abi.args();
        let first_arg_is_signer = arg_abis
            .first()
            .filter(|abi| abi.type_abi() == &TypeInstantiation::Signer)
            .is_some();
        if first_arg_is_signer {
            &arg_abis[1..]
        } else {
            arg_abis
        }
    };
    let args = arg_abis
        .iter()
        .zip(s.args())
        .map(|(ty, arg)| decode_move_value(ty.type_abi(), arg))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DecodedScript {
        code: s.code().to_vec(),
        ty_args: s.ty_args().to_vec(),
        args,
    })
}
pub fn decode_package(state: &dyn StateView, pkg: &Package) -> Result<DecodedPackage> {
    let module_cache = ModuleCache::new();
    {
        let modules = pkg
            .modules()
            .iter()
            .map(|m| CompiledModule::deserialize(m.code()))
            .collect::<Result<Vec<_>, _>>()?;
        for m in modules {
            module_cache.insert(m.self_id(), m);
        }
    }
    let resolver = ABIResolver::new_with_module_cache(state, module_cache);
    let decoded_init_script = if let Some(init_script) = pkg.init_script() {
        Some(decode_script_function_inner(&resolver, init_script)?)
    } else {
        None
    };

    Ok(DecodedPackage {
        package_address: pkg.package_address(),
        modules: pkg.modules().to_vec(),
        init_script: decoded_init_script,
    })
}
pub fn decode_script_function(
    state: &dyn StateView,
    sf: &ScriptFunction,
) -> Result<DecodedScriptFunction> {
    let resolver = ABIResolver::new(state);
    decode_script_function_inner(&resolver, sf)
}

fn decode_script_function_inner(
    resolver: &ABIResolver,
    sf: &ScriptFunction,
) -> Result<DecodedScriptFunction> {
    let func_abi =
        resolver.resolve_function_instantiation(sf.module(), sf.function(), sf.ty_args())?;
    let arg_abis = {
        let arg_abis = func_abi.args();
        let first_arg_is_signer = arg_abis
            .first()
            .filter(|abi| abi.type_abi() == &TypeInstantiation::Signer)
            .is_some();
        if first_arg_is_signer {
            &arg_abis[1..]
        } else {
            arg_abis
        }
    };
    let args = arg_abis
        .iter()
        .zip(sf.args())
        .map(|(abi, arg)| decode_move_value(abi.type_abi(), arg))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DecodedScriptFunction {
        module: sf.module().clone(),
        function: sf.function().to_owned(),
        ty_args: sf.ty_args().to_vec(),
        args,
    })
}
