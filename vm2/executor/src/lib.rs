// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod executor;

pub mod block_executor;

pub use block_executor::block_execute;
pub use executor::{
    do_execute_block_transactions, execute_readonly_function, validate_transaction,
};
