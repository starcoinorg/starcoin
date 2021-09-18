// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// How to call a particular Move script (aka. an "ABI").
use anyhow::Result;
use schemars::JsonSchema;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::file_format::AbilitySet;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_vm_types::value::{MoveStructLayout, MoveTypeLayout};
use std::fmt;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub struct ModuleABI {
    #[schemars(with = "String")]
    module_name: ModuleId,
    structs: Vec<StructABI>,
    script_functions: Vec<FunctionABI>,
}

impl ModuleABI {
    pub fn new(
        module_name: ModuleId,
        structs: Vec<StructABI>,
        script_functions: Vec<FunctionABI>,
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
    pub fn script_functions(&self) -> &[FunctionABI] {
        &self.script_functions
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub enum ScriptABI {
    TransactionScript(TransactionScriptABI),
    ScriptFunction(FunctionABI),
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

    pub fn ty_args(&self) -> &[TypeParameterABI] {
        match self {
            Self::TransactionScript(abi) => abi.ty_args(),
            Self::ScriptFunction(abi) => abi.ty_args(),
        }
    }

    pub fn args(&self) -> &[FunctionParameterABI] {
        match self {
            Self::TransactionScript(abi) => abi.args(),
            Self::ScriptFunction(abi) => abi.args(),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub struct TransactionScriptABI {
    /// The public name of the script.
    name: String,
    /// Some text comment.
    doc: String,
    /// The `code` value to set in the `Script` object.
    code: Vec<u8>,
    /// The names of the type arguments.
    ty_args: Vec<TypeParameterABI>,
    /// The description of regular arguments.
    args: Vec<FunctionParameterABI>,
}
impl TransactionScriptABI {
    pub fn new(
        name: String,
        doc: String,
        code: Vec<u8>,
        ty_args: Vec<TypeParameterABI>,
        args: Vec<FunctionParameterABI>,
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

    pub fn ty_args(&self) -> &[TypeParameterABI] {
        &self.ty_args
    }

    pub fn args(&self) -> &[FunctionParameterABI] {
        &self.args
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub struct FunctionABI {
    /// The public name of the script.
    name: String,
    /// The module name where the script lives.
    #[schemars(with = "String")]
    module_name: ModuleId,
    /// Some text comment.
    doc: String,
    /// The names of the type arguments.
    ty_args: Vec<TypeParameterABI>,
    /// The description of regular arguments.
    args: Vec<FunctionParameterABI>,
    /// return types
    returns: Vec<TypeInstantiation>,
}

impl FunctionABI {
    pub fn new(
        name: String,
        module_name: ModuleId,
        doc: String,
        ty_args: Vec<TypeParameterABI>,
        args: Vec<FunctionParameterABI>,
        returns: Vec<TypeInstantiation>,
    ) -> Self {
        Self {
            name,
            module_name,
            doc,
            ty_args,
            args,
            returns,
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

    pub fn ty_args(&self) -> &[TypeParameterABI] {
        &self.ty_args
    }

    pub fn args(&self) -> &[FunctionParameterABI] {
        &self.args
    }
    pub fn returns(&self) -> &[TypeInstantiation] {
        &self.returns
    }

    pub fn instantiation(&self, ty_args: &[TypeInstantiation]) -> Result<FunctionInstantiation> {
        let ty_params: Vec<_> = self
            .ty_args
            .iter()
            .zip(ty_args)
            .map(|(ty_arg, t)| ty_arg.instantiation(t.clone()))
            .collect();

        let args = self
            .args
            .iter()
            .map(|arg| arg.subst(ty_args))
            .collect::<Result<Vec<_>>>()?;
        let rets = self
            .returns
            .iter()
            .map(|ret| ret.subst(ty_args))
            .collect::<Result<Vec<_>>>()?;

        Ok(FunctionInstantiation {
            name: self.name.clone(),
            module_name: self.module_name.clone(),
            doc: self.doc.clone(),
            ty_args: ty_params,
            args,
            returns: rets,
        })
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub struct FunctionInstantiation {
    /// The public name of the script.
    name: String,
    /// The module name where the script lives.
    #[schemars(with = "String")]
    module_name: ModuleId,
    /// Some text comment.
    doc: String,
    /// The names of the type arguments.
    ty_args: Vec<TypeParameterInstantiation>,
    /// The description of regular arguments.
    args: Vec<FunctionParameterABI>,
    /// return types
    returns: Vec<TypeInstantiation>,
}
impl FunctionInstantiation {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn module_name(&self) -> &ModuleId {
        &self.module_name
    }

    pub fn doc(&self) -> &str {
        &self.doc
    }

    pub fn ty_args(&self) -> &[TypeParameterInstantiation] {
        &self.ty_args
    }

    pub fn args(&self) -> &[FunctionParameterABI] {
        &self.args
    }
    pub fn returns(&self) -> &[TypeInstantiation] {
        &self.returns
    }
}

/// The description of a (regular) argument in a script.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub struct FunctionParameterABI {
    /// The name of the argument.
    name: String,
    /// The expected type.
    /// In Move scripts, this does contain generics type parameters.
    type_tag: TypeInstantiation,
    /// The doc of the arg.
    doc: String,
}
impl FunctionParameterABI {
    pub fn new(name: String, type_abi: TypeInstantiation, doc: String) -> Self {
        Self {
            name,
            type_tag: type_abi,
            doc,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_abi(&self) -> &TypeInstantiation {
        &self.type_tag
    }
    pub fn doc(&self) -> &str {
        &self.doc
    }
    pub fn subst(&self, ty_args: &[TypeInstantiation]) -> Result<FunctionParameterABI> {
        let mut this = self.clone();
        this.type_tag = this.type_tag.subst(ty_args)?;
        Ok(this)
    }
}

/// The description of a type argument in a script.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub struct TypeParameterABI {
    /// The name of the argument.
    name: String,
    abilities: WrappedAbilitySet,
    phantom: bool,
}

impl TypeParameterABI {
    pub fn new(name: String, abilities: AbilitySet, phantom: bool) -> Self {
        Self {
            name,
            abilities: WrappedAbilitySet(abilities),
            phantom,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn ability_set(&self) -> AbilitySet {
        self.abilities.0
    }
    pub fn is_phantom(&self) -> bool {
        self.phantom
    }
    pub fn instantiation(&self, ty: TypeInstantiation) -> TypeParameterInstantiation {
        TypeParameterInstantiation {
            abi: self.clone(),
            ty,
        }
    }
}
/// The description of a type argument in a script.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub struct TypeParameterInstantiation {
    #[serde(flatten)]
    abi: TypeParameterABI,
    ty: TypeInstantiation,
}

impl TypeParameterInstantiation {
    pub fn abi(&self) -> &TypeParameterABI {
        &self.abi
    }
    pub fn ty(&self) -> &TypeInstantiation {
        &self.ty
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub struct StructABI {
    /// name of the struct
    name: String,
    /// module contains the struct
    #[schemars(with = "String")]
    module_name: ModuleId,
    ty_args: Vec<TypeParameterABI>,
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
        ty_args: Vec<TypeParameterABI>,
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

    /// instantiate the struct by passing the type arguments of the Struct/Function Definition.
    pub fn instantiations(&self, ty_args: &[TypeInstantiation]) -> Result<StructInstantiation> {
        Ok(StructInstantiation {
            name: self.name.clone(),
            module_name: self.module_name.clone(),
            doc: self.doc.clone(),
            abilities: self.abilities,
            ty_args: self
                .ty_args
                .iter()
                .zip(ty_args)
                .map(|(t, p)| t.instantiation(p.clone()))
                .collect(),
            fields: self
                .fields
                .iter()
                .map(|f| f.subst(ty_args))
                .collect::<Result<_>>()?,
        })
    }
}
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub struct StructInstantiation {
    /// name of the struct
    name: String,
    /// module contains the struct
    #[schemars(with = "String")]
    module_name: ModuleId,
    ty_args: Vec<TypeParameterInstantiation>,
    /// fields of the structs.
    fields: Vec<FieldABI>,
    /// The doc of the struct
    doc: String,
    abilities: WrappedAbilitySet,
}
impl StructInstantiation {
    pub fn new(
        name: String,
        module_name: ModuleId,
        doc: String,
        ty_args: Vec<TypeParameterInstantiation>,
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

    pub fn struct_tag(&self) -> Result<StructTag> {
        Ok(StructTag {
            address: *self.module_name.address(),
            module: self.module_name.name().to_owned(),
            name: Identifier::new(self.name.as_str())?,
            type_params: self
                .ty_args
                .iter()
                .map(|t| t.ty.type_tag())
                .collect::<Result<Vec<_>>>()?,
        })
    }

    /// Substitute all the type parameter in this type instantiation with given `ty_args`
    pub fn subst(&self, ty_args: &[TypeInstantiation]) -> Result<StructInstantiation> {
        Ok(Self {
            name: self.name.clone(),
            module_name: self.module_name.clone(),
            doc: self.doc.clone(),
            abilities: self.abilities,
            ty_args: self.ty_args.clone(),
            fields: self
                .fields
                .iter()
                .map(|f| f.subst(ty_args))
                .collect::<Result<_>>()?,
        })
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub struct FieldABI {
    /// field name
    name: String,
    /// doc of the field
    doc: String,
    /// type of the field
    type_abi: TypeInstantiation,
}
impl FieldABI {
    pub fn new(name: String, doc: String, type_abi: TypeInstantiation) -> Self {
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
    pub fn type_abi(&self) -> &TypeInstantiation {
        &self.type_abi
    }
    /// passing by the type arguments of StructDefinition or FunctionDefinition
    pub fn subst(&self, ty_args: &[TypeInstantiation]) -> Result<FieldABI> {
        Ok(Self {
            name: self.name.clone(),
            doc: self.doc.clone(),
            type_abi: self.type_abi.subst(ty_args)?,
        })
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub enum TypeInstantiation {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Signer,
    Vector(Box<TypeInstantiation>),
    Struct(Box<StructInstantiation>),
    TypeParameter(usize),
    Reference(/* mut */ bool, /* type */ Box<TypeInstantiation>),
}
impl TypeInstantiation {
    pub fn new_vector(subtype: TypeInstantiation) -> Self {
        Self::Vector(Box::new(subtype))
    }
    pub fn new_struct_instantiation(s: StructInstantiation) -> Self {
        Self::Struct(Box::new(s))
    }
    pub fn layout(&self) -> Result<MoveTypeLayout> {
        Ok(match self {
            Self::Bool => MoveTypeLayout::Bool,
            Self::U8 => MoveTypeLayout::U8,

            Self::U64 => MoveTypeLayout::U64,
            Self::U128 => MoveTypeLayout::U128,
            Self::Address => MoveTypeLayout::Address,
            Self::Signer => MoveTypeLayout::Signer,

            Self::Vector(t) => MoveTypeLayout::Vector(Box::new(t.layout()?)),
            Self::Struct(s) => MoveTypeLayout::Struct(s.layout()?),
            Self::TypeParameter(_) => anyhow::bail!("get type layout failed -- {:?}", self),
            Self::Reference(_, _) => anyhow::bail!("get type layout failed -- {:?}", self),
        })
    }
    pub fn type_tag(&self) -> Result<TypeTag> {
        Ok(match self {
            Self::Bool => TypeTag::Bool,
            Self::U8 => TypeTag::U8,

            Self::U64 => TypeTag::U64,
            Self::U128 => TypeTag::U128,
            Self::Address => TypeTag::Address,
            Self::Signer => TypeTag::Signer,

            Self::Vector(t) => TypeTag::Vector(Box::new(t.type_tag()?)),
            Self::Struct(s) => TypeTag::Struct(s.struct_tag()?),
            Self::TypeParameter(_) => anyhow::bail!("get type tag failed -- {:?}", self),
            Self::Reference(_, _) => anyhow::bail!("get type tag failed -- {:?}", self),
        })
    }
    pub fn subst(&self, ty_args: &[TypeInstantiation]) -> Result<TypeInstantiation> {
        use TypeInstantiation as T;
        Ok(match self {
            T::TypeParameter(idx) => match ty_args.get(*idx) {
                Some(ty) => ty.clone(),
                None => anyhow::bail!(
                    "type abi substitution failed: index out of bounds -- len {} got {}",
                    ty_args.len(),
                    idx
                ),
            },
            T::Vector(ty) => T::Vector(Box::new(ty.subst(ty_args)?)),
            T::Struct(struct_ty) => T::Struct(Box::new(struct_ty.subst(ty_args)?)),
            T::Reference(mutable, ty) => T::Reference(*mutable, Box::new(ty.subst(ty_args)?)),
            _ => self.clone(),
        })
    }
}

impl<'d> serde::de::DeserializeSeed<'d> for &TypeInstantiation {
    type Value = serde_json::Value;

    fn deserialize<D: serde::de::Deserializer<'d>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        use serde_json::Value as V;
        use TypeInstantiation as T;
        match &self {
            T::Bool => bool::deserialize(deserializer).map(V::Bool),
            T::U8 => u8::deserialize(deserializer).map(Into::into),
            T::U64 => u64::deserialize(deserializer).map(Into::into),
            T::U128 => u128::deserialize(deserializer).map(Into::into),
            T::Address => {
                AccountAddress::deserialize(deserializer).map(|addr| V::String(addr.to_string()))
            }
            T::Signer => {
                AccountAddress::deserialize(deserializer).map(|addr| V::String(addr.to_string()))
            }
            T::Struct(ty) => Ok(ty.as_ref().deserialize(deserializer)?),
            T::Vector(layout) => Ok(match layout.as_ref() {
                T::U8 => {
                    let bytes = <Vec<u8>>::deserialize(deserializer)?;
                    V::String(format!("0x{}", hex::encode(&bytes)))
                }
                _ => V::Array(deserializer.deserialize_seq(VectorElementVisitor(layout.as_ref()))?),
            }),
            T::TypeParameter(_) => Err(D::Error::custom(
                "type abi cannot be type parameter variant",
            )),
            T::Reference(_, _) => Err(D::Error::custom("type abi cannot be Reference variant")),
        }
    }
}

struct VectorElementVisitor<'a>(&'a TypeInstantiation);

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
        while let Some(elem) = seq.next_element_seed(self.0)? {
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
            match seq.next_element_seed(field_type.type_abi())? {
                Some(elem) => val.push(elem),
                None => return Err(A::Error::invalid_length(i, &self)),
            }
        }
        Ok(val)
    }
}

impl<'d> serde::de::DeserializeSeed<'d> for &StructInstantiation {
    type Value = serde_json::Value;

    fn deserialize<D: serde::de::Deserializer<'d>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        let layout = &self;
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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, JsonSchema)]
pub struct WrappedAbilitySet(#[schemars(with = "String")] pub AbilitySet);

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
