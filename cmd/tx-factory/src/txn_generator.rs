// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_executor::TransactionExecutor;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::authenticator::AuthenticationKey;
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
        MockTxnGenerator {
            receiver_address,
            receiver_auth_key_prefix,
            account,
        }
    }

    pub fn generate_mock_txn<E>(&self, sequence_number: u64) -> Result<RawUserTransaction>
    where
        E: TransactionExecutor + Sync + Send,
    {
        // TODO: make it configurable
        let amount_to_transfer = 1000;

        // let balance_resource = state_db.get_balance(self.account.address())?;
        // if balance_resource.is_none() || (balance_resource.unwrap() <= amount_to_transfer) {
        //     bail!("not enough balance, skip gen mock txn, please faucet it first");
        // }

        let transfer_txn = <E as TransactionExecutor>::build_transfer_txn(
            self.account.address,
            AuthenticationKey::ed25519(&self.account.public_key).to_vec(),
            self.receiver_address,
            self.receiver_auth_key_prefix.clone(),
            sequence_number,
            amount_to_transfer,
        );
        Ok(transfer_txn)
    }
}
