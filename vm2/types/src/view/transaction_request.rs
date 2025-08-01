use crate::view::{scripts_data::ScriptData, str_view::StrView, ByteCode};
use move_core_types::account_address::AccountAddress;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{CryptoMaterialError, ValidCryptoMaterialStringExt};
use starcoin_vm_types::transaction::{
    authenticator::AccountPublicKey, RawUserTransaction, TransactionPayload,
};
use std::str::FromStr;

#[derive(Default, Clone, Debug, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct TransactionRequest {
    /// Sender's address.
    #[schemars(with = "Option<String>")]
    pub sender: Option<AccountAddress>,
    // Sequence number of this transaction corresponding to sender's account.
    pub sequence_number: Option<u64>,
    /// The transaction script to execute.
    #[serde(default)]
    pub script: Option<ScriptData>,
    /// module codes.
    #[serde(default)]
    pub modules: Vec<StrView<ByteCode>>,
    // Maximal total gas specified by wallet to spend for this transaction.
    pub max_gas_amount: Option<u64>,
    // Maximal price can be paid per gas.
    pub gas_unit_price: Option<u64>,
    // The token code for pay transaction gas, Default is STC token code.
    pub gas_token_code: Option<String>,
    // Expiration timestamp for this transaction. timestamp is represented
    // as u64 in seconds from Unix Epoch. If storage is queried and
    // the time returned is greater than or equal to this time and this
    // transaction has not been included, you can be certain that it will
    // never be included.
    // A transaction that doesn't expire is represented by a very large value like
    // u64::max_value().
    pub expiration_timestamp_secs: Option<u64>,
    pub chain_id: Option<u8>,
}

impl From<RawUserTransaction> for TransactionRequest {
    fn from(raw: RawUserTransaction) -> Self {
        let mut request = Self {
            sender: Some(raw.sender()),
            sequence_number: Some(raw.sequence_number()),
            script: None,
            modules: vec![],
            max_gas_amount: Some(raw.max_gas_amount()),
            gas_unit_price: Some(raw.gas_unit_price()),
            gas_token_code: Some(raw.gas_token_code()),
            expiration_timestamp_secs: Some(raw.expiration_timestamp_secs()),
            chain_id: Some(raw.chain_id().id()),
        };
        match raw.into_payload() {
            TransactionPayload::Script(s) => {
                request.script = Some(s.into());
            }
            TransactionPayload::Package(p) => {
                let (_, m, s) = p.into_inner();
                request.script = s.map(Into::into);
                request.modules = m.into_iter().map(|m| StrView(m.into())).collect();
            }
            TransactionPayload::EntryFunction(s) => {
                request.script = Some(ScriptData::from(s));
            }
        }
        request
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct DryRunTransactionRequest {
    #[serde(flatten)]
    pub transaction: TransactionRequest,
    /// Sender's public key
    pub sender_public_key: StrView<AccountPublicKey>,
}

impl std::fmt::Display for StrView<Vec<u8>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl FromStr for StrView<Vec<u8>> {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(hex::decode(s.strip_prefix("0x").unwrap_or(s))?))
    }
}

impl std::fmt::Display for StrView<AccountPublicKey> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.to_encoded_string().map_err(|_| std::fmt::Error)?
        )
    }
}
impl FromStr for StrView<AccountPublicKey> {
    type Err = CryptoMaterialError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AccountPublicKey::from_encoded_string(s).map(StrView)
    }
}
