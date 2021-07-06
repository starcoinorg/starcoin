use crate::file_format::AbilitySet;
use crate::language_storage::ModuleId;

use anyhow::Result;
use move_core_types::value::{MoveStructLayout, MoveTypeLayout};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// How to call a particular Move script (aka. an "ABI").
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum ScriptABI {
    TransactionScript(TransactionScriptABI),
    ScriptFunction(ScriptFunctionABI),
}
impl ScriptABI {
    pub fn is_script_fun_abi(&self) -> bool {
        matches!(self, Self::ScriptFunction(_))
    }

    pub fn is_transaction_script_abi(&self) -> bool {
        matches!(self, Self::TransactionScript(_))
    }

    pub fn name(&self) -> &str {
        match self {
            Self::TransactionScript(abi) => abi.name(),
            Self::ScriptFunction(abi) => abi.name(),
        }
    }

    pub fn doc(&self) -> &str {
        match self {
            Self::TransactionScript(abi) => abi.doc(),
            Self::ScriptFunction(abi) => abi.doc(),
        }
    }

    pub fn ty_args(&self) -> &[TypeArgumentABI] {
        match self {
            Self::TransactionScript(abi) => abi.ty_args(),
            Self::ScriptFunction(abi) => abi.ty_args(),
        }
    }

    pub fn args(&self) -> &[ArgumentABI] {
        match self {
            Self::TransactionScript(abi) => abi.args(),
            Self::ScriptFunction(abi) => abi.args(),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct ScriptFunctionABI {
    /// The public name of the script.
    name: String,
    /// The module name where the script lives.
    module_name: ModuleId,
    /// Some text comment.
    doc: String,
    /// The names of the type arguments.
    ty_args: Vec<TypeArgumentABI>,
    /// The description of regular arguments.
    args: Vec<ArgumentABI>,
}
impl ScriptFunctionABI {
    pub fn new(
        name: String,
        module_name: ModuleId,
        doc: String,
        ty_args: Vec<TypeArgumentABI>,
        args: Vec<ArgumentABI>,
    ) -> Self {
        Self {
            name,
            module_name,
            doc,
            ty_args,
            args,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn module_name(&self) -> &ModuleId {
        &self.module_name
    }

    pub fn doc(&self) -> &str {
        &self.doc
    }

    pub fn ty_args(&self) -> &[TypeArgumentABI] {
        &self.ty_args
    }

    pub fn args(&self) -> &[ArgumentABI] {
        &self.args
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct TransactionScriptABI {
    /// The public name of the script.
    name: String,
    /// Some text comment.
    doc: String,
    /// The `code` value to set in the `Script` object.
    #[serde(with = "serde_bytes")]
    code: Vec<u8>,
    /// The names of the type arguments.
    ty_args: Vec<TypeArgumentABI>,
    /// The description of regular arguments.
    args: Vec<ArgumentABI>,
}
impl TransactionScriptABI {
    pub fn new(
        name: String,
        doc: String,
        code: Vec<u8>,
        ty_args: Vec<TypeArgumentABI>,
        args: Vec<ArgumentABI>,
    ) -> Self {
        Self {
            name,
            doc,
            code,
            ty_args,
            args,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn doc(&self) -> &str {
        &self.doc
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn ty_args(&self) -> &[TypeArgumentABI] {
        &self.ty_args
    }

    pub fn args(&self) -> &[ArgumentABI] {
        &self.args
    }
}

/// The description of a (regular) argument in a script.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct ArgumentABI {
    /// The name of the argument.
    name: String,
    /// The expected type.
    /// In Move scripts, this does contain generics type parameters.
    type_tag: TypeABI,
    /// The doc of the arg.
    doc: String,
}
impl ArgumentABI {
    pub fn new(name: String, type_abi: TypeABI, doc: String) -> Self {
        Self {
            name,
            type_tag: type_abi,
            doc,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_abi(&self) -> &TypeABI {
        &self.type_tag
    }
    pub fn doc(&self) -> &str {
        &self.doc
    }
}

/// The description of a type argument in a script.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct TypeArgumentABI {
    /// The name of the argument.
    name: String,
    abilities: WrappedAbilitySet,
}
impl TypeArgumentABI {
    pub fn new(name: String, abilities: AbilitySet) -> Self {
        Self {
            name,
            abilities: WrappedAbilitySet(abilities),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn ability_set(&self) -> AbilitySet {
        self.abilities.0
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum TypeABI {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Signer,
    Vector(Box<TypeABI>),
    Struct(Box<StructABI>),
    TypeParameter(usize),
}
impl TypeABI {
    pub fn new_vector(subtype: TypeABI) -> Self {
        TypeABI::Vector(Box::new(subtype))
    }
    pub fn new_struct(s: StructABI) -> Self {
        TypeABI::Struct(Box::new(s))
    }
    pub fn layout(&self) -> Result<MoveTypeLayout> {
        Ok(match self {
            TypeABI::Bool => MoveTypeLayout::Bool,
            TypeABI::U8 => MoveTypeLayout::U8,

            TypeABI::U64 => MoveTypeLayout::U64,
            TypeABI::U128 => MoveTypeLayout::U128,
            TypeABI::Address => MoveTypeLayout::Address,
            TypeABI::Signer => MoveTypeLayout::Signer,

            TypeABI::Vector(t) => MoveTypeLayout::Vector(Box::new(t.layout()?)),
            TypeABI::Struct(s) => MoveTypeLayout::Struct(s.layout()?),
            TypeABI::TypeParameter(_) => anyhow::bail!("get type layout failed -- {:?}", self),
        })
    }
    pub fn subst(&self, ty_args: &[TypeABI]) -> Result<TypeABI> {
        use TypeABI::*;
        Ok(match self {
            TypeParameter(idx) => match ty_args.get(*idx) {
                Some(ty) => ty.clone(),
                None => anyhow::bail!(
                    "type abi substitution failed: index out of bounds -- len {} got {}",
                    ty_args.len(),
                    idx
                ),
            },

            Bool => Bool,
            U8 => U8,
            U64 => U64,
            U128 => U128,
            Address => Address,
            Signer => Signer,
            Vector(ty) => Vector(Box::new(ty.subst(ty_args)?)),
            Struct(struct_ty) => Struct(Box::new(struct_ty.subst(ty_args)?)),
        })
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct StructABI {
    /// name of the struct
    name: String,
    /// module contains the struct
    module_name: ModuleId,
    ty_args: Vec<TypeArgumentABI>,
    /// fields of the structs.
    fields: Vec<FieldABI>,
    /// The doc of the struct
    doc: String,
    abilities: WrappedAbilitySet,
}
impl StructABI {
    pub fn new(
        name: String,
        module_name: ModuleId,
        doc: String,
        ty_args: Vec<TypeArgumentABI>,
        fields: Vec<FieldABI>,
        abilities: AbilitySet,
    ) -> Self {
        Self {
            name,
            module_name,
            ty_args,
            doc,
            fields,
            abilities: WrappedAbilitySet(abilities),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn doc(&self) -> &str {
        &self.doc
    }
    pub fn fields(&self) -> &[FieldABI] {
        &self.fields
    }
    pub fn module_name(&self) -> &ModuleId {
        &self.module_name
    }
    pub fn ability_set(&self) -> AbilitySet {
        self.abilities.0
    }
    pub fn layout(&self) -> Result<MoveStructLayout> {
        let fs = self
            .fields
            .iter()
            .map(|f| f.type_abi.layout())
            .collect::<Result<Vec<_>>>()?;
        Ok(MoveStructLayout::new(fs))
    }
    pub fn subst(&self, ty_args: &[TypeABI]) -> Result<StructABI> {
        Ok(Self {
            name: self.name.clone(),
            module_name: self.module_name.clone(),
            doc: self.doc.clone(),
            abilities: self.abilities,
            ty_args: self.ty_args.clone(),
            fields: self
                .fields
                .iter()
                .map(|f| {
                    Ok(FieldABI::new(
                        f.name.clone(),
                        f.doc.clone(),
                        f.type_abi.subst(ty_args)?,
                    ))
                })
                .collect::<Result<_>>()?,
        })
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct FieldABI {
    /// field name
    name: String,
    /// doc of the field
    doc: String,
    /// type of the field
    type_abi: TypeABI,
}
impl FieldABI {
    pub fn new(name: String, doc: String, type_abi: TypeABI) -> Self {
        Self {
            name,
            doc,
            type_abi,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn doc(&self) -> &str {
        &self.doc
    }
    pub fn type_abi(&self) -> &TypeABI {
        &self.type_abi
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct ModuleABI {
    module_name: ModuleId,
    structs: Vec<StructABI>,
    script_functions: Vec<ScriptFunctionABI>,
}

impl ModuleABI {
    pub fn new(
        module_name: ModuleId,
        structs: Vec<StructABI>,
        script_functions: Vec<ScriptFunctionABI>,
    ) -> Self {
        Self {
            module_name,
            structs,
            script_functions,
        }
    }
    pub fn module_name(&self) -> &ModuleId {
        &self.module_name
    }
    pub fn structs(&self) -> &[StructABI] {
        &self.structs
    }
    pub fn script_functions(&self) -> &[ScriptFunctionABI] {
        &self.script_functions
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct WrappedAbilitySet(pub AbilitySet);

impl Serialize for WrappedAbilitySet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.into_u8().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WrappedAbilitySet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let byte = u8::deserialize(deserializer)?;
        Ok(WrappedAbilitySet(AbilitySet::from_u8(byte).ok_or_else(
            || serde::de::Error::custom(format!("Invalid ability set: {:X}", byte)),
        )?))
    }
}
