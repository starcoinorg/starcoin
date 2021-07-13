// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_binary_format::CompiledModule;
use starcoin_abi_resolver::ABIResolver;
use starcoin_resource_viewer::module_cache::ModuleCache;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::{ModuleId, TypeTag};
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::{Module, ScriptFunction, TransactionPayload};
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
    pub args: Vec<serde_json::Value>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedScriptFunction {
    pub module: ModuleId,
    pub function: Identifier,
    pub ty_args: Vec<TypeTag>,
    pub args: Vec<serde_json::Value>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedPackage {
    ///Package's all Module must at same address.
    pub package_address: AccountAddress,
    pub modules: Vec<Module>,
    pub init_script: Option<DecodedScriptFunction>,
}

pub fn decode_txn_payload(
    state: &dyn StateView,
    payload: &TransactionPayload,
) -> Result<DecodedTransactionPayload> {
    match payload {
        TransactionPayload::Script(s) => {
            let resolver = ABIResolver::new(state);
            let script_abi = resolver.resolve_script(s.code().to_vec())?;
            let args = s
                .args()
                .iter()
                .zip(script_abi.args())
                .map(|(arg, ty)| bcs::from_bytes_seed(ty.type_abi(), arg))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(DecodedScript {
                code: s.code().to_vec(),
                ty_args: s.ty_args().to_vec(),
                args,
            }
            .into())
        }
        TransactionPayload::Package(pkg) => {
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
                Some(decode_script_function(&resolver, init_script)?)
            } else {
                None
            };

            Ok(DecodedPackage {
                package_address: pkg.package_address(),
                modules: pkg.modules().to_vec(),
                init_script: decoded_init_script,
            }
            .into())
        }
        TransactionPayload::ScriptFunction(sf) => {
            let resolver = ABIResolver::new(state);
            Ok(decode_script_function(&resolver, sf)?.into())
        }
    }
}

fn decode_script_function(
    resolver: &ABIResolver,
    sf: &ScriptFunction,
) -> Result<DecodedScriptFunction> {
    let func_abi =
        resolver.resolve_function_instantiation(sf.module(), sf.function(), sf.ty_args())?;
    let args = func_abi
        .args()
        .iter()
        .zip(sf.args())
        .map(|(abi, arg)| bcs::from_bytes_seed(abi.type_abi(), arg))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DecodedScriptFunction {
        module: sf.module().clone(),
        function: sf.function().to_owned(),
        ty_args: sf.ty_args().to_vec(),
        args,
    })
}
