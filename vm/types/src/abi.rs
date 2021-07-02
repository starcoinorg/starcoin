use crate::language_storage::ModuleId;
use anyhow::Result;
use serde::{Deserialize, Serialize};
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
}

/// The description of a type argument in a script.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct TypeArgumentABI {
    /// The name of the argument.
    name: String,
}
impl TypeArgumentABI {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
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
    Struct(StructABI),
    TypeParameter(usize),
}
impl TypeABI {
    pub fn new_vector(subtype: TypeABI) -> Self {
        TypeABI::Vector(Box::new(subtype))
    }
    pub fn new_struct(s: StructABI) -> Self {
        TypeABI::Struct(s)
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
            Struct(struct_ty) => Struct(struct_ty.subst(ty_args)?),
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
    /// fields of the structs.
    fields: Vec<FieldABI>,
    /// The doc of the struct
    doc: String,
}
impl StructABI {
    pub fn new(name: String, module_name: ModuleId, doc: String, fields: Vec<FieldABI>) -> Self {
        Self {
            name,
            module_name,
            doc,
            fields,
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

    pub fn subst(&self, ty_args: &[TypeABI]) -> Result<StructABI> {
        Ok(Self {
            name: self.name.clone(),
            module_name: self.module_name.clone(),
            doc: self.doc.clone(),
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
    type_abi: Box<TypeABI>,
}
impl FieldABI {
    pub fn new(name: String, doc: String, type_abi: TypeABI) -> Self {
        Self {
            name,
            doc,
            type_abi: Box::new(type_abi),
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
    module_id: ModuleId,
    structs: Vec<StructABI>,
    script_functions: Vec<ScriptFunctionABI>,
}

impl ModuleABI {
    pub fn new(
        module_id: ModuleId,
        structs: Vec<StructABI>,
        script_functions: Vec<ScriptFunctionABI>,
    ) -> Self {
        Self {
            module_id,
            structs,
            script_functions,
        }
    }
    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }
    pub fn structs(&self) -> &[StructABI] {
        &self.structs
    }
    pub fn script_functions(&self) -> &[ScriptFunctionABI] {
        &self.script_functions
    }
}
