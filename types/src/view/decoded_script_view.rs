// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::function_arg_type_view::TypeTagView;
use crate::view::str_view::StrView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DecodedScriptView {
    pub code: StrView<Vec<u8>>,
    pub ty_args: Vec<TypeTagView>,
    pub args: Vec<DecodedMoveValue>,
}

impl From<DecodedScript> for DecodedScriptView {
    fn from(orig: DecodedScript) -> Self {
        Self {
            code: StrView(orig.code),
            ty_args: orig.ty_args.into_iter().map(StrView).collect(),
            args: orig.args,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DecodedPackageView {
    pub package_address: AccountAddress,
    pub modules: Vec<StrView<Vec<u8>>>,
    pub init_script: Option<DecodedScriptFunctionView>,
}

impl From<DecodedPackage> for DecodedPackageView {
    fn from(orig: DecodedPackage) -> Self {
        Self {
            package_address: orig.package_address,
            modules: orig
                .modules
                .into_iter()
                .map(|m| StrView(m.code().to_vec()))
                .collect(),
            init_script: orig.init_script.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DecodedScriptFunctionView {
    pub module: ModuleIdView,
    #[schemars(with = "String")]
    pub function: Identifier,
    pub ty_args: Vec<TypeTagView>,
    pub args: Vec<DecodedMoveValue>,
}

impl From<DecodedScriptFunction> for DecodedScriptFunctionView {
    fn from(orig: DecodedScriptFunction) -> Self {
        Self {
            module: StrView(orig.module),
            function: orig.function,
            ty_args: orig.ty_args.into_iter().map(StrView).collect(),
            args: orig.args,
        }
    }
}
