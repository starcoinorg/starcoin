// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_vm2_account_api::AccountInfo;
use starcoin_vm2_types::view::{
    DryRunOutputView, RawUserTransactionView, TransactionEventResponse, TransactionInfoView,
};
use starcoin_vm2_vm_types::account_config::token_code::TokenCode;
use std::collections::HashMap;

#[derive(Serialize, Debug, Clone)]
pub struct ExecutionOutputView {
    pub txn_hash: HashValue,
    pub txn_info: Option<TransactionInfoView>,
    pub events: Option<Vec<TransactionEventResponse>>,
}
impl ExecutionOutputView {
    pub fn new(txn_hash: HashValue) -> Self {
        Self {
            txn_hash,
            txn_info: None,
            events: None,
        }
    }

    pub fn new_with_info(
        txn_hash: HashValue,
        txn_info: TransactionInfoView,
        events: Vec<TransactionEventResponse>,
    ) -> Self {
        Self {
            txn_hash,
            txn_info: Some(txn_info),
            events: Some(events),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExecuteResultView {
    pub raw_txn: RawUserTransactionView,
    pub raw_txn_hex: String,
    pub dry_run_output: DryRunOutputView,
    pub execute_output: Option<ExecutionOutputView>,
}

impl ExecuteResultView {
    pub fn new(
        raw_txn: RawUserTransactionView,
        raw_txn_hex: String,
        dry_run_output: DryRunOutputView,
    ) -> Self {
        Self {
            raw_txn,
            raw_txn_hex,
            dry_run_output,
            execute_output: None,
        }
    }
    pub fn get_transaction_info(&self) -> Option<TransactionInfoView> {
        if let Some(info) = &self.execute_output {
            info.txn_info.clone()
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountWithStateView {
    pub account: AccountInfo,
    pub auth_key: String,
    pub sequence_number: Option<u64>,
    pub balances: HashMap<TokenCode, u128>,
}
