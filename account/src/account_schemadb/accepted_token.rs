use super::AccountAddressWrapper;
use bcs_ext::BCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_schemadb::{
    define_schema,
    error::{StoreError, StoreResult},
    schema::{KeyCodec, ValueCodec},
    ColumnFamilyName,
};
use starcoin_types::account_config::token_code::TokenCode;

pub const ACCEPTED_TOKEN_PREFIX_NAME: ColumnFamilyName = "accepted_token";

define_schema!(
    AcceptedToken,
    AccountAddressWrapper,
    AcceptedTokens,
    ACCEPTED_TOKEN_PREFIX_NAME
);

impl KeyCodec<AcceptedToken> for AccountAddressWrapper {
    fn encode_key(&self) -> StoreResult<Vec<u8>> {
        Ok(self.0.to_vec())
    }

    fn decode_key(data: &[u8]) -> StoreResult<Self> {
        AccountAddressWrapper::try_from(data).map_err(StoreError::DecodeError)
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct AcceptedTokens(pub Vec<TokenCode>);

impl ValueCodec<AcceptedToken> for AcceptedTokens {
    fn encode_value(&self) -> StoreResult<Vec<u8>> {
        self.0
            .encode()
            .map_err(|e| StoreError::EncodeError(e.to_string()))
    }

    fn decode_value(data: &[u8]) -> StoreResult<Self> {
        <Vec<TokenCode>>::decode(data)
            .map(AcceptedTokens)
            .map_err(|e| StoreError::DecodeError(e.to_string()))
    }
}
