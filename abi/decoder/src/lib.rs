use anyhow::Result;
use move_binary_format::CompiledModule;
use serde::de::Error;
use serde::Deserialize;
use serde::Serialize;
use starcoin_resource_viewer::abi_resolver::ABIResolver;
use starcoin_resource_viewer::module_cache::ModuleCache;
use starcoin_vm_types::abi::{FieldABI, StructABI, TypeABI};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::{ModuleId, TypeTag};
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::{Module, ScriptFunction, TransactionPayload};
use std::fmt;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct WrappedTypeABI(TypeABI);

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
struct WrappedStructABI(StructABI);

impl<'d> serde::de::DeserializeSeed<'d> for &WrappedTypeABI {
    type Value = serde_json::Value;

    fn deserialize<D: serde::de::Deserializer<'d>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        use serde_json::Value as V;
        use TypeABI as ABI;
        match &self.0 {
            ABI::Bool => bool::deserialize(deserializer).map(V::Bool),
            ABI::U8 => u8::deserialize(deserializer).map(Into::into),
            ABI::U64 => u64::deserialize(deserializer).map(Into::into),
            ABI::U128 => u128::deserialize(deserializer).map(Into::into),
            ABI::Address => {
                AccountAddress::deserialize(deserializer).map(|addr| V::String(addr.to_string()))
            }
            ABI::Signer => {
                AccountAddress::deserialize(deserializer).map(|addr| V::String(addr.to_string()))
            }
            ABI::Struct(ty) => Ok(WrappedStructABI(ty.as_ref().clone()).deserialize(deserializer)?),
            ABI::Vector(layout) => Ok(match layout.as_ref() {
                TypeABI::U8 => {
                    let bytes = <Vec<u8>>::deserialize(deserializer)?;
                    match String::from_utf8(bytes) {
                        Ok(s) => V::String(s),
                        Err(e) => V::String(format!("0x{}", hex::encode(e.as_bytes()))),
                    }
                }
                _ => V::Array(deserializer.deserialize_seq(VectorElementVisitor(layout.as_ref()))?),
            }),
            TypeABI::TypeParameter(_) => Err(D::Error::custom(
                "type abi cannot be type parameter variant",
            )),
        }
    }
}

struct VectorElementVisitor<'a>(&'a TypeABI);

impl<'d, 'a> serde::de::Visitor<'d> for VectorElementVisitor<'a> {
    type Value = Vec<serde_json::Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Vector")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'d>,
    {
        let mut vals = Vec::new();
        while let Some(elem) = seq.next_element_seed(&WrappedTypeABI(self.0.clone()))? {
            vals.push(elem)
        }
        Ok(vals)
    }
}

struct StructFieldVisitor<'a>(&'a [FieldABI]);

impl<'d, 'a> serde::de::Visitor<'d> for StructFieldVisitor<'a> {
    type Value = Vec<serde_json::Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Struct")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'d>,
    {
        let mut val = Vec::new();
        for (i, field_type) in self.0.iter().enumerate() {
            match seq.next_element_seed(&WrappedTypeABI(field_type.type_abi().clone()))? {
                Some(elem) => val.push(elem),
                None => return Err(A::Error::invalid_length(i, &self)),
            }
        }
        Ok(val)
    }
}

impl<'d> serde::de::DeserializeSeed<'d> for &WrappedStructABI {
    type Value = serde_json::Value;

    fn deserialize<D: serde::de::Deserializer<'d>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        let layout = &self.0;
        let fields = deserializer
            .deserialize_tuple(layout.fields().len(), StructFieldVisitor(layout.fields()))?;
        let fields: serde_json::Map<_, _> = layout
            .fields()
            .iter()
            .map(|f| f.name().to_string())
            .zip(fields)
            .collect();
        Ok(serde_json::Value::Object(fields))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecodedScript {
    #[serde(with = "serde_bytes")]
    pub code: Vec<u8>,
    pub ty_args: Vec<TypeTag>,
    pub args: Vec<serde_json::Value>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecodedScriptFunction {
    pub module: ModuleId,
    pub function: Identifier,
    pub ty_args: Vec<TypeTag>,
    pub args: Vec<serde_json::Value>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
                .map(|(arg, ty)| bcs::from_bytes_seed(&WrappedTypeABI(ty.type_abi().clone()), arg))
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
        .map(|(abi, arg)| bcs::from_bytes_seed(&WrappedTypeABI(abi.type_abi().clone()), arg))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DecodedScriptFunction {
        module: sf.module().clone(),
        function: sf.function().to_owned(),
        ty_args: sf.ty_args().to_vec(),
        args,
    })
}
