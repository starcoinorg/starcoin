// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{
    arguments_view::ArgumentsView, byte_code_or_script_function::ByteCodeOrScriptFunction,
    function_arg_type_view::TypeTagView, str_view::StrView,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::{
    language_storage::FunctionId,
    transaction::{EntryFunction, Script, TransactionPayload},
};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct ScriptData {
    pub code: StrView<ByteCodeOrScriptFunction>,
    #[serde(default)]
    pub type_args: Vec<TypeTagView>,
    pub args: ArgumentsView,
}

impl ScriptData {
    pub fn into_script_function(self) -> anyhow::Result<EntryFunction> {
        match self.into_data() {
            Err(script_function) => Ok(script_function),
            _ => {
                anyhow::bail!("not a script function");
            }
        }
    }
    pub fn into_script(self) -> anyhow::Result<Script> {
        match self.into_data() {
            Ok(script) => Ok(script),
            _ => {
                anyhow::bail!("not a script");
            }
        }
    }
    fn into_data(self) -> Result<Script, EntryFunction> {
        let ty_args: Vec<_> = self.type_args.into_iter().map(|s| s.0).collect();
        let args: Vec<_> = self.args.to_bcs_bytes();

        match self.code.0 {
            ByteCodeOrScriptFunction::ByteCode(code) => Ok(Script::new(code, ty_args, args)),
            ByteCodeOrScriptFunction::ScriptFunction(FunctionId { module, function }) => {
                Err(EntryFunction::new(module, function, ty_args, args))
            }
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<TransactionPayload> for ScriptData {
    fn into(self) -> TransactionPayload {
        match self.into_data() {
            Ok(script) => TransactionPayload::Script(script),
            Err(func) => TransactionPayload::EntryFunction(func),
        }
    }
}

impl From<Script> for ScriptData {
    fn from(s: Script) -> Self {
        let (code, ty_args, args) = s.into_inner();
        Self {
            code: StrView(ByteCodeOrScriptFunction::ByteCode(code)),
            type_args: ty_args.into_iter().map(TypeTagView::from).collect(),
            args: ArgumentsView::BCS(args.into_iter().map(StrView).collect()),
        }
    }
}

impl From<EntryFunction> for ScriptData {
    fn from(s: EntryFunction) -> Self {
        let (module, function, ty_args, args) = s.into_inner();
        Self {
            code: StrView(ByteCodeOrScriptFunction::ScriptFunction(FunctionId {
                module,
                function,
            })),
            type_args: ty_args.into_iter().map(TypeTagView::from).collect(),
            args: ArgumentsView::BCS(args.into_iter().map(StrView).collect()),
        }
    }
}
