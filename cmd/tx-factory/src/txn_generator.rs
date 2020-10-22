// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_account_api::AccountInfo;
use starcoin_crypto::ed25519::Ed25519PublicKey;
use starcoin_executor::DEFAULT_EXPIRATION_TIME;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::genesis_config::ChainId;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_types::transaction::helpers::get_current_timestamp;
use starcoin_types::transaction::RawUserTransaction;

pub struct MockTxnGenerator {
    chain_id: ChainId,
    receiver_address: AccountAddress,
    receiver_public_key: Option<Ed25519PublicKey>,
    account: AccountInfo,
}

impl MockTxnGenerator {
    pub fn new(
        chain_id: ChainId,
        account: AccountInfo,
        receiver_address: AccountAddress,
        receiver_public_key: Option<Ed25519PublicKey>,
    ) -> Self {
        MockTxnGenerator {
            chain_id,
            receiver_address,
            receiver_public_key,
            account,
        }
    }

    pub fn generate_mock_txn(&self, sequence_number: u64) -> Result<RawUserTransaction> {
        let amount_to_transfer = 1000;

        let transfer_txn = starcoin_executor::build_transfer_txn(
            self.account.address,
            self.receiver_address,
            self.receiver_public_key
                .as_ref()
                .map(|k| AuthenticationKey::ed25519(k)),
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
