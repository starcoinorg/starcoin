// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::AccountInfo;
use anyhow::Result;
use starcoin_service_registry::ServiceRequest;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::sign_message::{SignedMessage, SigningMessage};
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum AccountRequest {
    CreateAccount(String),
    GetDefaultAccount(),
    SetDefaultAccount(AccountAddress),
    RemoveAccount(AccountAddress, Option<String>),
    GetAccounts(),
    GetAccount(AccountAddress),
    SignTxn {
        txn: Box<RawUserTransaction>,
        signer: AccountAddress,
    },
    SignMessage {
        signer: AccountAddress,
        message: SigningMessage,
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
    ImportReadonlyAccount {
        address: AccountAddress,
        public_key: Vec<u8>,
    },
    ExportAccount {
        address: AccountAddress,
        password: String,
    },
    ChangePassword {
        address: AccountAddress,
        new_password: String,
    },
}

impl ServiceRequest for AccountRequest {
    type Response = Result<AccountResponse>;
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
    SignedMessage(Box<SignedMessage>),
    None,
}
