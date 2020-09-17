// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#[macro_use]
extern crate log;

pub use block_executor::{block_execute, BlockExecutedData};
pub use executor::*;
pub use starcoin_transaction_builder::{
    build_accept_token_txn, build_transfer_from_association, build_transfer_txn,
    build_transfer_txn_by_token_type, create_signed_txn_with_association_account,
    encode_create_account_script, encode_transfer_script, peer_to_peer_txn_sent_as_association,
    DEFAULT_EXPIRATION_TIME, DEFAULT_MAX_GAS_AMOUNT,
};

mod block_executor;
#[cfg(test)]
pub mod error_code_test;
mod executor;
#[cfg(test)]
pub mod executor_test;
#[cfg(test)]
pub mod readonly_function_call_test;
