// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::MyWorld;
use anyhow::Error;
use cucumber::{Steps, StepsBuilder};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::{RpcClient, StateRootOption};
use starcoin_transaction_builder::vm2::{
    build_transfer_txn, DEFAULT_EXPIRATION_TIME, DEFAULT_MAX_GAS_AMOUNT,
};
use starcoin_vm2_account_api::AccountInfo;
use starcoin_vm2_state_api::StateReaderExt;
use starcoin_vm2_types::{
    account_address::AccountAddress, account_config, transaction::SignedUserTransaction,
};
use starcoin_vm2_vm_types::{
    account_config::G_STC_TOKEN_CODE, genesis_config::ChainId, transaction::RawUserTransaction,
};
use std::time::Duration;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .then("charge money to account", |world: &mut MyWorld, _step| {
            let client = world
                .default_rpc_client
                .as_ref()
                .take()
                .expect("RPC Client not set");
            let to = world
                .default_account
                .as_ref()
                .take()
                .expect("RPC Account not set");
            let pre_mine_address = account_config::association_address();
            let result = transfer_txn(client, to, pre_mine_address, None);
            assert!(result.is_ok());
            std::thread::sleep(Duration::from_millis(3000));
            let chain_state_reader = client
                .state_reader2(StateRootOption::Latest)
                .expect("state reader");
            let balances = chain_state_reader.get_balance_by_type(
                *to.address(),
                G_STC_TOKEN_CODE
                    .clone()
                    .try_into()
                    .expect("Should convert 0x1::starcoin_coin::STC"),
            );
            assert!(balances.is_ok());
            info!(
                "charge into default account ok:{:?}",
                balances.expect("get balances failed")
            );
        })
        .then(
            "execute transfer transaction",
            |world: &mut MyWorld, _step| {
                let client = world
                    .default_rpc_client
                    .as_ref()
                    .take()
                    .expect("RPC Client not set");
                let from_account = world
                    .default_account
                    .as_ref()
                    .take()
                    .expect("RPC Account not set");
                let to_account = world
                    .txn_account
                    .as_ref()
                    .take()
                    .expect("Transaction Account not set");
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
    let chain_state_reader = client.state_reader2(StateRootOption::Latest)?;
    let account_resource = chain_state_reader.get_account_resource(from)?;
    let node_info = client.node_info()?;
    let balance = chain_state_reader.get_balance_by_type(
        from,
        G_STC_TOKEN_CODE
            .clone()
            .try_into()
            .expect("Should convert 0x1::starcoin_coin::STC"),
    )?;
    let amount = amount.unwrap_or(balance * 20 / 100);
    let raw_txn = build_transfer_txn(
        from,
        to.address,
        account_resource.sequence_number(),
        amount,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        node_info.now_seconds + DEFAULT_EXPIRATION_TIME,
        ChainId::new(node_info.net.chain_id().id()),
    );

    let txn = sign_txn(client, raw_txn)?;
    client.submit_transaction2(txn.clone())
}
fn sign_txn(
    client: &RpcClient,
    raw_txn: RawUserTransaction,
) -> Result<SignedUserTransaction, Error> {
    client.account_unlock2(raw_txn.sender(), "".to_string(), Duration::from_secs(300))?;
    client.account_sign_txn2(raw_txn)
}
