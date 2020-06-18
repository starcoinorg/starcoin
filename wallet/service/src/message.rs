// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::clock::Duration;
use actix::Message;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_wallet_api::{WalletAccount, WalletResult};

#[derive(Debug, Clone)]
pub enum WalletRequest {
    CreateAccount(String),
    GetDefaultAccount(),
    GetAccounts(),
    GetAccount(AccountAddress),
    SignTxn {
        txn: Box<RawUserTransaction>,
        signer: AccountAddress,
    },
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
    type Result = WalletResult<WalletResponse>;
}

#[derive(Debug, Clone)]
pub enum WalletResponse {
    WalletAccount(Box<WalletAccount>),
    WalletAccountOption(Box<Option<WalletAccount>>),
    AccountList(Vec<WalletAccount>),
    SignedTxn(Box<SignedUserTransaction>),
    UnlockAccountResponse,
    ExportAccountResponse(Vec<u8>),
    None,
}
