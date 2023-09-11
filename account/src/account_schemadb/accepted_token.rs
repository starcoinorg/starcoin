use anyhow::Result;
use bcs_ext::BCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_schemadb::{
    define_schema,
    schema::{KeyCodec, ValueCodec},
    ColumnFamilyName,
};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;

pub const ACCEPTED_TOKEN_PREFIX_NAME: ColumnFamilyName = "accepted_token";

define_schema!(
    AcceptedToken,
    AccountAddress,
    AcceptedTokens,
    ACCEPTED_TOKEN_PREFIX_NAME
);

impl KeyCodec<AcceptedToken> for AccountAddress {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
        AccountAddress::try_from(data).map_err(Into::into)
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct AcceptedTokens(pub Vec<TokenCode>);

impl ValueCodec<AcceptedToken> for AcceptedTokens {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.0.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        <Vec<TokenCode>>::decode(data).map(AcceptedTokens)
    }
}
