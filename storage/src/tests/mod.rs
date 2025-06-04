use starcoin_crypto::HashValue;
use starcoin_types::transaction::{StcRichTransactionInfo, TransactionInfo};
use starcoin_types::vm_error::KeptVMStatus;

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
mod test_accumulator;
mod test_batch;
mod test_block;
mod test_storage;

fn random_txn_info2(block_number: u64, gas_used: u64) -> StcRichTransactionInfo {
    StcRichTransactionInfo::new(
        HashValue::random(),
        block_number,
        TransactionInfo::new(
            HashValue::random(),
            HashValue::zero(),
            vec![].as_slice(),
            gas_used,
            KeptVMStatus::Executed,
        )
        .into(),
        rand::random(),
        rand::random(),
    )
}
fn random_txn_info(gas_used: u64) -> StcRichTransactionInfo {
    random_txn_info2(rand::random(), gas_used)
}
