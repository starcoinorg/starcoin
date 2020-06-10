// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::MyWorld;
use anyhow::Error;
use cucumber::{Steps, StepsBuilder};
use starcoin_logger::prelude::*;
use starcoin_rpc_client::{RemoteStateReader, RpcClient};
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_types::transaction::helpers::TransactionSigner;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_vm_runtime::common_transactions::TXN_RESERVED;
use starcoin_wallet_api::WalletAccount;
use std::time::Duration;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .then("charge money to account", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let to = world.default_account.as_ref().take().unwrap();
            let pre_mine_address = account_config::association_address();
            let result = transfer_txn(client, to, pre_mine_address, None);
            assert!(result.is_ok());
            std::thread::sleep(Duration::from_millis(3000));
            let chain_state_reader = RemoteStateReader::new(client);
            let account_state_reader = AccountStateReader::new(&chain_state_reader);
            let balances = account_state_reader.get_balances(to.address());
            assert!(balances.is_ok());
            info!("charge into default account ok:{:?}", balances.unwrap());
        })
        .then(
            "execute transfer transaction",
            |world: &mut MyWorld, _step| {
                let client = world.rpc_client.as_ref().take().unwrap();
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
    to: &WalletAccount,
    from: AccountAddress,
    amount: Option<u64>,
) -> Result<(), Error> {
    let to_auth_key_prefix = AuthenticationKey::ed25519(&to.public_key).prefix();
    let chain_state_reader = RemoteStateReader::new(client);
    let account_state_reader = AccountStateReader::new(&chain_state_reader);
    let account_resource = account_state_reader
        .get_account_resource(&from)
        .unwrap()
        .unwrap();
    let balance = account_state_reader.get_balance(&from).unwrap().unwrap();
    let amount = amount.unwrap_or(balance * 20 / 100);
    let raw_txn = starcoin_executor::build_transfer_txn(
        from,
        to.address,
        to_auth_key_prefix.to_vec(),
        account_resource.sequence_number(),
        amount,
        1,
        TXN_RESERVED,
    );

    let txn = sign_txn(client, raw_txn).unwrap();
    client.submit_transaction(txn.clone()).and_then(|r| r)
}
fn sign_txn(
    client: &RpcClient,
    raw_txn: RawUserTransaction,
) -> Result<SignedUserTransaction, Error> {
    let net = client.node_info().unwrap().net;
    let result = if raw_txn.sender() == account_config::association_address() {
        let chain_config = net.get_config();
        let pre_mine_config = chain_config
            .pre_mine_config
            .as_ref()
            .expect("Dev net pre mine config must exist.");
        pre_mine_config.sign_txn(raw_txn).unwrap()
    } else {
        client.wallet_sign_txn(raw_txn).unwrap()
    };
    Ok(result)
}
