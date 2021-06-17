// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_account_api::AccountInfo;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::genesis_config::ChainId;
use starcoin_types::transaction::RawUserTransaction;

pub struct MockTxnGenerator {
    chain_id: ChainId,
    receiver_address: AccountAddress,
    account: AccountInfo,
}

impl MockTxnGenerator {
    pub fn new(chain_id: ChainId, account: AccountInfo, receiver_address: AccountAddress) -> Self {
        MockTxnGenerator {
            chain_id,
            receiver_address,
            account,
        }
    }

    pub fn generate_mock_txn(
        &self,
        sequence_number: u64,
        expiration_timestamp: u64,
    ) -> Result<RawUserTransaction> {
        let amount_to_transfer = 1000;

        let transfer_txn = starcoin_executor::build_transfer_txn(
            self.account.address,
            self.receiver_address,
            sequence_number,
            amount_to_transfer,
            1,
            40000000,
            expiration_timestamp,
            self.chain_id,
        );
        Ok(transfer_txn)
    }

    pub fn generate_transfer_txn(
        &self,
        sequence_number: u64,
        sender: AccountAddress,
        receiver_address: AccountAddress,
        amount: u128,
        gas_price: u64,
        expiration_timestamp: u64,
    ) -> Result<RawUserTransaction> {
        let transfer_txn = starcoin_executor::build_transfer_txn(
            sender,
            receiver_address,
            sequence_number,
            amount,
            gas_price,
            40000000,
            expiration_timestamp,
            self.chain_id,
        );
        Ok(transfer_txn)
    }

    pub fn generate_account_txn(
        &self,
        sequence_number: u64,
        sender: AccountAddress,
        receiver_address_vec: Vec<AccountAddress>,
        amount: u128,
        gas_price: u64,
        expiration_timestamp: u64,
    ) -> Result<RawUserTransaction> {
        let transfer_txn = starcoin_executor::build_batch_transfer_txn(
            sender,
            receiver_address_vec,
            sequence_number,
            amount,
            gas_price,
            40000000,
            expiration_timestamp,
            self.chain_id,
        );
        Ok(transfer_txn)
    }
}
