// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use starcoin_executor::TransactionExecutor;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::{AccountAddress, AuthenticationKey};
use starcoin_types::transaction::RawUserTransaction;
use starcoin_wallet_api::WalletAccount;

pub struct MockTxnGenerator {
    receiver_address: AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,

    account: WalletAccount,
}

impl MockTxnGenerator {
    pub fn new(
        account: WalletAccount,
        receiver_address: AccountAddress,
        receiver_auth_key_prefix: Vec<u8>,
    ) -> Self {
        // let (prikey, pubkey) = Self::gen_key_pair();
        // let (b_pri, b_pub) = Self::gen_key_pair();
        MockTxnGenerator {
            receiver_address,
            receiver_auth_key_prefix,
            account,
        }
    }

    pub fn generate_mock_txn<E>(&self, state_db: &AccountStateReader) -> Result<RawUserTransaction>
    where
        E: TransactionExecutor + Sync + Send,
    {
        let account_resource = state_db.get_account_resource(self.account.address())?;
        if account_resource.is_none() {
            bail!(
                "account {} not exists, please faucet it",
                self.account.address()
            );
        }
        let account_resource = account_resource.unwrap();

        // TODO: make it configurable
        let amount_to_transfer = 1000;

        let balance_resource = state_db.get_balance(self.account.address())?;
        if balance_resource.is_none() || (balance_resource.unwrap() <= amount_to_transfer) {
            bail!("not enough balance, skip gen mock txn, please faucet it first");
        }

        let transfer_txn = <E as TransactionExecutor>::build_transfer_txn(
            self.account.address,
            AuthenticationKey::from_public_key(&self.account.public_key).to_vec(),
            self.receiver_address,
            self.receiver_auth_key_prefix.clone(),
            account_resource.sequence_number(),
            amount_to_transfer,
        );
        Ok(transfer_txn)
    }
}
