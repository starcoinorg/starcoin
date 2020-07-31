use anyhow::Result;
use futures::Stream;
use serde::Deserialize;
use serde::Serialize;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::contract_event::EventWithProof;
use starcoin_types::event::EventKey;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction, TransactionPayload};
use std::time::Duration;

pub trait ChainEventWatcher {
    fn watch_event<S>(&self, key: EventKey) -> S
    where
        S: Stream<Item = EventWithProof>;
}

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
    default_expiration_timeout: u64,
    default_gas_price: u64,
    default_gas_token: TokenCode,
}

pub trait WalletStorageTrait {
    fn save_default_settings(&self, setting: Setting) -> Result<()>;

    fn save_accepted_token(&self, token: TokenCode) -> Result<()>;
    fn contain_wallet(&self, address: AccountAddress) -> Result<bool>;
}
