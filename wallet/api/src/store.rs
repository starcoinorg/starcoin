// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::WalletAccount;
use anyhow::Result;
use starcoin_types::account_address::AccountAddress;

pub trait WalletStore {
    fn get_account(&self, address: &AccountAddress) -> Result<Option<WalletAccount>>;
    fn save_account(&self, account: WalletAccount) -> Result<()>;
    fn remove_account(&self, address: &AccountAddress) -> Result<()>;
    fn get_accounts(&self) -> Result<Vec<WalletAccount>>;
    fn save_to_account(&self, address: &AccountAddress, key: String, value: Vec<u8>) -> Result<()>;
    fn get_from_account(&self, address: &AccountAddress, key: &str) -> Result<Option<Vec<u8>>>;
}
