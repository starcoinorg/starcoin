// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::MyWorld;
use anyhow::Error;
use cucumber::{Steps, StepsBuilder};
use starcoin_executor::executor::Executor;
use starcoin_executor::TransactionExecutor;
use starcoin_rpc_client::{RemoteStateReader, RpcClient};
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_types::transaction::helpers::TransactionSigner;
use starcoin_wallet_api::WalletAccount;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .then("charge money to account", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let to = world.default_account.as_ref().take().unwrap();
            let pre_mine_address = account_config::association_address();
            let result = transfer_txn(client, to, pre_mine_address, None);
            assert!(result.is_ok());
        })
        .then(
            "execute transfer transaction",
            |world: &mut MyWorld, _step| {
                let client = world.rpc_client.as_ref().take().unwrap();
                let from_account = world.default_account.as_ref().take().unwrap();
                let to_account = world.txn_account.as_ref().take().unwrap();
                let result = transfer_txn(client, to_account, from_account.address, Some(100));
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
) -> Result<bool, Error> {
    let chain_config = client.node_info().unwrap().net.get_config();
    let pre_mine_config = chain_config
        .pre_mine_config
        .as_ref()
        .expect("Dev net pre mine config must exist.");

    let to_auth_key_prefix = AuthenticationKey::ed25519(&to.public_key).prefix();

    let chain_state_reader = RemoteStateReader::new(client);
    let account_state_reader = AccountStateReader::new(&chain_state_reader);
    let account_resource = account_state_reader
        .get_account_resource(&from)
        .unwrap()
        .unwrap();
    let balance = account_state_reader.get_balance(&from).unwrap().unwrap();
    let amount = amount.unwrap_or(balance * 20 / 100);
    let raw_txn = Executor::build_transfer_txn(
        from,
        to.address,
        to_auth_key_prefix.to_vec(),
        account_resource.sequence_number(),
        amount,
    );
    let txn = pre_mine_config.sign_txn(raw_txn).unwrap();
    client.submit_transaction(txn.clone())
}
