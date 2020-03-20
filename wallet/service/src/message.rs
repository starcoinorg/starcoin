// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::Message;
use anyhow::Result;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_wallet_api::WalletAccount;

#[derive(Debug, Clone)]
pub enum WalletRequest {
    CreateAccount(String),
    GetDefaultAccount(),
    GetAccounts(),
    SignTxn(RawUserTransaction),
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
    None,
}
