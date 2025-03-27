// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "testing")]
#[macro_use]
extern crate log;

pub use block_executor::{
    block_execute, get_logger_balance_amount, set_logger_balance_amount_once, BlockExecutedData,
};
pub use executor::*;
// pub use starcoin_transaction_builder::{
//     build_accept_token_txn, build_batch_transfer_txn, build_transfer_from_association,
//     build_transfer_txn, build_transfer_txn_by_token_type,
//     create_signed_txn_with_association_account, encode_create_account_script_function,
//     encode_nft_transfer_script, encode_transfer_script_by_token_code,
//     encode_transfer_script_function, peer_to_peer_txn_sent_as_association, DEFAULT_EXPIRATION_TIME,
//     DEFAULT_MAX_GAS_AMOUNT,
// };
pub use starcoin_vm_runtime::metrics::VMMetrics;

mod block_executor;

mod executor;
mod executor2;
