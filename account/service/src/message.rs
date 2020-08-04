// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::Message;
use starcoin_account_api::{AccountInfo, AccountResult};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum AccountRequest {
    CreateAccount(String),
    GetDefaultAccount(),
    GetAccounts(),
    GetAccount(AccountAddress),
    SignTxn {
        txn: Box<RawUserTransaction>,
        signer: AccountAddress,
    },
    AccountAcceptedTokens {
        address: AccountAddress,
    },
    UnlockAccount(AccountAddress, String, Duration),
    LockAccount(AccountAddress),
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

impl Message for AccountRequest {
    type Result = AccountResult<AccountResponse>;
}

#[derive(Debug, Clone)]
pub enum AccountResponse {
    AccountInfo(Box<AccountInfo>),
    AccountInfoOption(Box<Option<AccountInfo>>),
    AccountList(Vec<AccountInfo>),
    SignedTxn(Box<SignedUserTransaction>),
    UnlockAccountResponse,
    ExportAccountResponse(Vec<u8>),
    AcceptedTokens(Vec<TokenCode>),
    None,
}
