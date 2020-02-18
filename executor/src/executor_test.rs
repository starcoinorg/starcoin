// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//use super::*;
use crate::{
    mock_executor::{
        MockChainState,  MockExecutor, encode_mint_transaction, encode_transfer_transaction, DISCARD_STATUS, KEEP_STATUS,
    },
    TransactionExecutor,
};
use config::VMConfig;
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
};

fn gen_address(index: u8) -> AccountAddress {
    AccountAddress::new([index; ADDRESS_LENGTH])
}

#[test]
fn test_execute_txn() {
    let chain_state = MockChainState::new();
    let txn = encode_mint_transaction(gen_address(0), 100);
    let config = VMConfig::default();

    let output = MockExecutor::execute_transaction(&config, &chain_state, txn).unwrap();

    assert_eq!(
        KEEP_STATUS.clone(),
        *output.status()
    );
}
