// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_executor::DEFAULT_EXPIRATION_TIME;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::chain_config::ChainId;
use starcoin_types::transaction::helpers::get_current_timestamp;
use starcoin_types::transaction::RawUserTransaction;
use starcoin_wallet_api::WalletAccount;

pub struct MockTxnGenerator {
    chain_id: ChainId,
    receiver_address: AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    account: WalletAccount,
}

impl MockTxnGenerator {
    pub fn new(
        chain_id: ChainId,
        account: WalletAccount,
        receiver_address: AccountAddress,
        receiver_auth_key_prefix: Vec<u8>,
    ) -> Self {
        MockTxnGenerator {
            chain_id,
            receiver_address,
            receiver_auth_key_prefix,
            account,
        }
    }

    pub fn generate_mock_txn(&self, sequence_number: u64) -> Result<RawUserTransaction> {
        let amount_to_transfer = 1000;

        let transfer_txn = starcoin_executor::build_transfer_txn(
            self.account.address,
            self.receiver_address,
            self.receiver_auth_key_prefix.clone(),
            sequence_number,
            amount_to_transfer,
            1,
            10000,
            get_current_timestamp() + DEFAULT_EXPIRATION_TIME,
            self.chain_id,
        );
        Ok(transfer_txn)
    }
}
