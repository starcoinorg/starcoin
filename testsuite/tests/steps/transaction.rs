// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::MyWorld;
use anyhow::Error;
use cucumber::{Steps, StepsBuilder};
use starcoin_account_api::AccountInfo;
use starcoin_crypto::HashValue;
use starcoin_executor::{DEFAULT_EXPIRATION_TIME, DEFAULT_MAX_GAS_AMOUNT};
use starcoin_logger::prelude::*;
use starcoin_rpc_client::{RpcClient, StateRootOption};
use starcoin_state_api::StateReaderExt;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::time::Duration;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .then("charge money to account", |world: &mut MyWorld, _step| {
            let client = world.default_rpc_client.as_ref().take().unwrap();
            let to = world.default_account.as_ref().take().unwrap();
            let pre_mine_address = account_config::association_address();
            let result = transfer_txn(client, to, pre_mine_address, None);
            assert!(result.is_ok());
            std::thread::sleep(Duration::from_millis(3000));
            let chain_state_reader = client.state_reader(StateRootOption::Latest).unwrap();
            let balances = chain_state_reader.get_balance(*to.address());
            assert!(balances.is_ok());
            info!("charge into default account ok:{:?}", balances.unwrap());
        })
        .then(
            "execute transfer transaction",
            |world: &mut MyWorld, _step| {
                let client = world.default_rpc_client.as_ref().take().unwrap();
                let from_account = world.default_account.as_ref().take().unwrap();
                let to_account = world.txn_account.as_ref().take().unwrap();
                info!("transfer from: {:?} to: {:?}", from_account, to_account);
                let result = transfer_txn(client, to_account, from_account.address, Some(1000));
                assert!(result.is_ok());
            },
        );
    builder.build()
}

fn transfer_txn(
    client: &RpcClient,
    to: &AccountInfo,
    from: AccountAddress,
    amount: Option<u128>,
) -> Result<HashValue, Error> {
    let chain_state_reader = client.state_reader(StateRootOption::Latest)?;
    let account_resource = chain_state_reader
        .get_account_resource(from)
        .unwrap()
        .unwrap();
    let node_info = client.node_info()?;
    let balance = chain_state_reader.get_balance(from).unwrap().unwrap();
    let amount = amount.unwrap_or(balance * 20 / 100);
    let raw_txn = starcoin_executor::build_transfer_txn(
        from,
        to.address,
        account_resource.sequence_number(),
        amount,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        node_info.now_seconds + DEFAULT_EXPIRATION_TIME,
        node_info.net.chain_id(),
    );

    let txn = sign_txn(client, raw_txn).unwrap();
    client.submit_transaction(txn.clone())
}
fn sign_txn(
    client: &RpcClient,
    raw_txn: RawUserTransaction,
) -> Result<SignedUserTransaction, Error> {
    client.account_unlock(raw_txn.sender(), "".to_string(), Duration::from_secs(300))?;
    Ok(client.account_sign_txn(raw_txn).unwrap())
}
