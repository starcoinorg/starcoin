use anyhow::Result;
use serde::de::{MapAccess, Visitor};
use serde::Serialize;
use serde::{Deserialize, Deserializer};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::account_config::STC_TOKEN_CODE;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction, TransactionPayload};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;
use std::time::Duration;

pub trait TransactionSubmitter {
    fn submit_transaction(&self, txn: SignedUserTransaction) -> Result<()>;
}

pub trait RichWallet {
    fn set_default_expiration_timeout(&mut self, timeout: Duration);
    fn set_default_gas_price(&mut self, gas_price: u64);
    fn set_default_gas_token(&mut self, token: TokenCode);

    fn get_accepted_tokns(&self) -> Result<Vec<TokenCode>>;

    fn build_transaction(
        &self,
        // if not specified, use default sender.
        sender: Option<AccountAddress>,
        payload: TransactionPayload,
        max_gas_amount: u64,
        // if not specified, uses default settings.
        gas_unit_price: Option<u64>,
        gas_token_code: Option<String>,
        expiration_timestamp_secs: Option<u64>,
    ) -> Result<RawUserTransaction>;

    fn sign_transaction(
        &self,
        raw: RawUserTransaction,
        address: Option<AccountAddress>,
    ) -> Result<SignedUserTransaction>;
    fn submit_txn(&mut self, txn: SignedUserTransaction) -> Result<()>;
    fn get_next_available_seq_number(&self) -> Result<u64>;

    // ...other functionality of origin wallets.
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub default_expiration_timeout: u64,
    pub default_gas_price: u64,
    #[serde(deserialize_with = "token_string_or_struct")]
    pub default_gas_token: TokenCode,
    /// this account is default account.
    pub is_default: bool,
    /// this account is readonly
    pub is_readonly: bool,
}

impl Setting {
    pub fn default() -> Self {
        Setting {
            default_expiration_timeout: 3600,
            default_gas_price: 1,
            default_gas_token: STC_TOKEN_CODE.clone(),
            is_default: false,
            is_readonly: false,
        }
    }

    pub fn readonly() -> Self {
        Setting {
            default_expiration_timeout: 3600,
            default_gas_price: 1,
            default_gas_token: STC_TOKEN_CODE.clone(),
            is_default: false,
            is_readonly: true,
        }
    }
}

fn token_string_or_struct<'de, D>(deserializer: D) -> Result<TokenCode, D::Error>
where
    D: Deserializer<'de>,
{
    struct TokenCodeStringOrStruct(PhantomData<fn() -> TokenCode>);

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TokenCodeV {
        pub address: AccountAddress,
        pub module: String,
        pub name: String,
    }

    impl<'de> Visitor<'de> for TokenCodeStringOrStruct {
        type Value = TokenCode;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string")
        }

        fn visit_str<E>(self, value: &str) -> Result<TokenCode, E>
        where
            E: serde::de::Error,
        {
            TokenCode::from_str(value).map_err(serde::de::Error::custom)
        }

        fn visit_map<M>(self, map: M) -> Result<TokenCode, M::Error>
        where
            M: MapAccess<'de>,
        {
            let token_code_v: TokenCodeV =
                Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
            Ok(TokenCode::new(
                token_code_v.address,
                token_code_v.module,
                token_code_v.name,
            ))
        }
    }
    deserializer.deserialize_any(TokenCodeStringOrStruct(PhantomData))
}

pub trait WalletStorageTrait {
    fn save_default_settings(&self, setting: Setting) -> Result<()>;

    fn save_accepted_token(&self, token: TokenCode) -> Result<()>;
    fn contain_wallet(&self, address: AccountAddress) -> Result<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_serialize() {
        let setting = Setting {
            ..Setting::default()
        };
        let json = serde_json::to_string(&setting).unwrap();
        println!("{}", json);
        let setting2: Setting = serde_json::from_str(json.as_str()).unwrap();
        assert_eq!(setting, setting2);

        let old_json = serde_json::json!({
            "default_expiration_timeout":3600,
            "default_gas_price":1,
            "default_gas_token":{
                "address": "0x1",
                "module": "STC",
                "name": "STC",
            },
            "is_default":false,
            "is_readonly":false}
        );
        let setting3: Setting = serde_json::from_value(old_json).unwrap();
        assert_eq!(setting, setting3);
    }
}
