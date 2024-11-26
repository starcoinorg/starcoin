// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::core_code_address;
use crate::{serde_helper::vec_bytes, transaction::user_transaction_context::EntryFunctionPayload};
use bcs_ext::Sample;
pub use move_core_types::abi::{
    ArgumentABI, ScriptFunctionABI as EntryFunctionABI, TransactionScriptABI, TypeArgumentABI,
};
use move_core_types::identifier::{IdentStr, Identifier};
use move_core_types::language_storage::{ModuleId, TypeTag};
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use std::fmt;

/// How to call a particular Move script (aka. an "ABI"). This is a clone of
/// move_core_types::abi::ScriptABI but with a tweak on EntryFunction -> EntryFunction
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum EntryABI {
    TransactionScript(TransactionScriptABI),
    EntryFunction(EntryFunctionABI),
}

impl EntryABI {
    pub fn is_entry_fun_abi(&self) -> bool {
        matches!(self, Self::EntryFunction(_))
    }

    pub fn is_transaction_script_abi(&self) -> bool {
        matches!(self, Self::TransactionScript(_))
    }

    pub fn name(&self) -> &str {
        match self {
            Self::TransactionScript(abi) => abi.name(),
            Self::EntryFunction(abi) => abi.name(),
        }
    }

    pub fn doc(&self) -> &str {
        match self {
            Self::TransactionScript(abi) => abi.doc(),
            Self::EntryFunction(abi) => abi.doc(),
        }
    }

    pub fn ty_args(&self) -> &[TypeArgumentABI] {
        match self {
            Self::TransactionScript(abi) => abi.ty_args(),
            Self::EntryFunction(abi) => abi.ty_args(),
        }
    }

    pub fn args(&self) -> &[ArgumentABI] {
        match self {
            Self::TransactionScript(abi) => abi.args(),
            Self::EntryFunction(abi) => abi.args(),
        }
    }
}

/// Call a Move script.
#[derive(Clone, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Script {
    #[serde(with = "serde_bytes")]
    #[schemars(with = "String")]
    code: Vec<u8>,
    #[schemars(with = "Vec<String>")]
    ty_args: Vec<TypeTag>,
    #[serde(with = "vec_bytes")]
    #[schemars(with = "Vec<String>")]
    args: Vec<Vec<u8>>,
}

impl Script {
    pub fn new(code: Vec<u8>, ty_args: Vec<TypeTag>, args: Vec<Vec<u8>>) -> Self {
        Self {
            code,
            ty_args,
            args,
        }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn ty_args(&self) -> &[TypeTag] {
        &self.ty_args
    }

    pub fn args(&self) -> &[Vec<u8>] {
        &self.args
    }

    pub fn into_inner(self) -> (Vec<u8>, Vec<TypeTag>, Vec<Vec<u8>>) {
        (self.code, self.ty_args, self.args)
    }
}

impl fmt::Debug for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Script")
            .field("code", &hex::encode(&self.code))
            .field("ty_args", &self.ty_args)
            .field("args", &self.args)
            .finish()
    }
}

impl Sample for Script {
    /// Sample script source code empty_script.move
    fn sample() -> Self {
        Self {
            code: hex::decode("a11ceb0b0100000001050001000000000102")
                .expect("Decode sample script should success."),
            ty_args: vec![],
            args: vec![],
        }
    }
}

impl Sample for EntryFunction {
    fn sample() -> Self {
        Self {
            module: ModuleId::new(
                core_code_address(),
                Identifier::new("empty_scripts").unwrap(),
            ),
            function: Identifier::new("empty_script").unwrap(),
            ty_args: vec![],
            args: vec![],
        }
    }
}

/// Call a Move entry function.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EntryFunction {
    #[schemars(with = "String")]
    module: ModuleId,
    #[schemars(with = "String")]
    function: Identifier,
    #[schemars(with = "Vec<String>")]
    ty_args: Vec<TypeTag>,
    #[serde(with = "vec_bytes")]
    #[schemars(with = "Vec<String>")]
    args: Vec<Vec<u8>>,
}

impl EntryFunction {
    pub fn new(
        module: ModuleId,
        function: Identifier,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    ) -> Self {
        EntryFunction {
            module,
            function,
            ty_args,
            args,
        }
    }

    pub fn module(&self) -> &ModuleId {
        &self.module
    }

    pub fn function(&self) -> &IdentStr {
        &self.function
    }

    pub fn ty_args(&self) -> &[TypeTag] {
        &self.ty_args
    }

    pub fn args(&self) -> &[Vec<u8>] {
        &self.args
    }

    pub fn into_inner(self) -> (ModuleId, Identifier, Vec<TypeTag>, Vec<Vec<u8>>) {
        (self.module, self.function, self.ty_args, self.args)
    }

    pub fn as_entry_function_payload(&self) -> EntryFunctionPayload {
        EntryFunctionPayload::new(
            self.module.address,
            self.module.name().to_string(),
            self.function.to_string(),
            self.ty_args.iter().map(|ty| ty.to_string()).collect(),
            self.args.clone(),
        )
    }
}
