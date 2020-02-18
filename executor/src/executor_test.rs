// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//use super::*;
use crate::{
    mock_executor::{
        encode_mint_transaction, encode_transfer_transaction, MockExecutor, DISCARD_STATUS, KEEP_STATUS,
    },
};

#[test]
fn test_execute_txn() {
    let chain_state = MockChainState::new();
    let txn = encode_mint_transaction(gen_address(0), 100);
    let config = VMConfig::default();

    let output = MockExecutor::execute_transaction(&config, &chain_state, txn)?;

    assert_eq!(
        KEEP_STATUS.clone(),
        output.status()
    );
}
