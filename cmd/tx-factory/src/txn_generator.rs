// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_account_api::AccountInfo;
use starcoin_transaction_builder::vm2::build_batch_transfer_txn as build_batch_transfer_txn2;
use starcoin_transaction_builder::vm2::build_transfer_txn as build_transfer_txn2;
use starcoin_transaction_builder::{build_batch_transfer_txn, build_transfer_txn};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::genesis_config::ChainId;
use starcoin_types::transaction::RawUserTransaction;
use starcoin_vm2_account_api::AccountInfo as AccountInfo2;
use starcoin_vm2_vm_types::{
    account_address::AccountAddress as AccountAddress2,
    transaction::RawUserTransaction as RawUserTransaction2,
};

pub struct MockTxnGenerator {
    chain_id: ChainId,
    receiver_address: AccountAddress,
    account: AccountInfo,

    receiver_address2: AccountAddress2,
    account2: AccountInfo2,
}

impl MockTxnGenerator {
    pub fn new(
        chain_id: ChainId,
        account: AccountInfo,
        receiver_address: AccountAddress,
        account2: AccountInfo2,
        receiver_address2: AccountAddress2,
    ) -> Self {
        Self {
            chain_id,
            receiver_address,
            account,
            receiver_address2,
            account2,
        }
    }

    pub fn generate_mock_txn2(
        &self,
        sequence_number: u64,
        expiration_timestamp: u64,
    ) -> Result<RawUserTransaction2> {
        let amount_to_transfer = 1000;

        let transfer_txn = build_transfer_txn2(
            self.account2.address,
            self.receiver_address2,
            sequence_number,
            amount_to_transfer,
            1,
            4000000,
            expiration_timestamp,
            self.chain_id.id().into(),
        );
        Ok(transfer_txn)
    }

    pub fn generate_mock_txn(
        &self,
        sequence_number: u64,
        expiration_timestamp: u64,
    ) -> Result<RawUserTransaction> {
        let amount_to_transfer = 1000;

        let transfer_txn = build_transfer_txn(
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

    pub fn generate_transfer_txn2(
        &self,
        sequence_number: u64,
        sender: AccountAddress2,
        receiver_address: AccountAddress2,
        amount: u128,
        gas_price: u64,
        expiration_timestamp: u64,
    ) -> Result<RawUserTransaction2> {
        let transfer_txn = build_transfer_txn2(
            sender,
            receiver_address,
            sequence_number,
            amount,
            gas_price,
            40_000_000,
            expiration_timestamp,
            self.chain_id.id().into(),
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
        let transfer_txn = build_transfer_txn(
            sender,
            receiver_address,
            sequence_number,
            amount,
            gas_price,
            4000000,
            expiration_timestamp,
            self.chain_id,
        );
        Ok(transfer_txn)
    }

    pub fn generate_account_txn2(
        &self,
        sequence_number: u64,
        sender: AccountAddress2,
        receiver_address_vec: Vec<AccountAddress2>,
        amount: u128,
        gas_price: u64,
        expiration_timestamp: u64,
    ) -> Result<RawUserTransaction2> {
        let transfer_txn = build_batch_transfer_txn2(
            sender,
            receiver_address_vec,
            sequence_number,
            amount,
            gas_price,  // 1 -1000
            40_000_000, // no more than 40_000_000
            expiration_timestamp,
            self.chain_id.id().into(),
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
        let transfer_txn = build_batch_transfer_txn(
            sender,
            receiver_address_vec,
            sequence_number,
            amount,
            gas_price,
            4000000,
            expiration_timestamp,
            self.chain_id,
        );
        Ok(transfer_txn)
    }
}
