// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::clock::Duration;
use actix::Message;
use anyhow::Result;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_wallet_api::WalletAccount;

#[derive(Debug, Clone)]
pub enum WalletRequest {
    CreateAccount(String),
    GetDefaultAccount(),
    GetAccounts(),
    GetAccount(AccountAddress),
    SignTxn(RawUserTransaction),
    UnlockAccount(AccountAddress, String, Duration),
    ImportAccount {
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    },
    ExportAccount {
        address: AccountAddress,
        password: String,
    },
}

impl Message for WalletRequest {
    type Result = Result<WalletResponse>;
}

#[derive(Debug, Clone)]
pub enum WalletResponse {
    WalletAccount(WalletAccount),
    WalletAccountOption(Option<WalletAccount>),
    AccountList(Vec<WalletAccount>),
    SignedTxn(SignedUserTransaction),
    Account(Option<WalletAccount>),
    UnlockAccountResponse,
    ImportAccountResponse(WalletAccount),
    ExportAccountResponse(Vec<u8>),
    None,
}
