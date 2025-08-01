use crate::view::decoded_script_view::{
    DecodedPackageView, DecodedScriptFunctionView, DecodedScriptView,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_abi_decoder::DecodedTransactionPayload;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TransactionPayloadView {
    /// A transaction that executes code.
    Script(DecodedScriptView),
    /// A transaction that publish or update module code by a package.
    Package(DecodedPackageView),
    /// A transaction that executes an existing script function published on-chain.
    ScriptFunction(DecodedScriptFunctionView),
}

impl From<DecodedTransactionPayload> for TransactionPayloadView {
    fn from(orig: DecodedTransactionPayload) -> Self {
        match orig {
            DecodedTransactionPayload::Script(s) => Self::Script(s.into()),
            DecodedTransactionPayload::Package(p) => Self::Package(p.into()),
            DecodedTransactionPayload::ScriptFunction(s) => Self::ScriptFunction(s.into()),
        }
    }
}
