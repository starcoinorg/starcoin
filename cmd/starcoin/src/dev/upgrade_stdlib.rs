// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use anyhow::{bail, format_err, Result};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::transaction::RawUserTransaction;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::association_address;
use starcoin_vm_types::genesis_config::ChainNetwork;
use starcoin_vm_types::transaction::{SignedUserTransaction, TransactionPayload};

pub fn sign_txn_with_association_account_by_rpc_client(
    cli_state: &CliState,
    max_gas_amount: u64,
    gas_price: u64,
    expiration_time: u64,
    payload: TransactionPayload,
) -> Result<SignedUserTransaction> {
    sign_txn_by_rpc_client(
        cli_state,
        max_gas_amount,
        gas_price,
        expiration_time,
        payload,
        Some(association_address()),
    )
}

pub fn sign_txn_with_default_account_by_rpc_client(
    cli_state: &CliState,
    max_gas_amount: u64,
    gas_price: u64,
    expiration_time: u64,
    payload: TransactionPayload,
) -> Result<SignedUserTransaction> {
    sign_txn_by_rpc_client(
        cli_state,
        max_gas_amount,
        gas_price,
        expiration_time,
        payload,
        None,
    )
}

fn sign_txn_by_rpc_client(
    cli_state: &CliState,
    max_gas_amount: u64,
    gas_price: u64,
    expiration_time: u64,
    payload: TransactionPayload,
    account_address: Option<AccountAddress>,
) -> Result<SignedUserTransaction> {
    let account = cli_state.get_account_or_default(account_address)?;
    let client = cli_state.client();
    let node_info = client.node_info()?;
    let chain_state_reader = RemoteStateReader::new(client);
    let account_state_reader = AccountStateReader::new(&chain_state_reader);
    let account_resource = account_state_reader
        .get_account_resource(account.address())?
        .ok_or_else(|| format_err!("account must exist on chain."))?;
    let net = ChainNetwork::new_builtin(
        *cli_state
            .net()
            .as_builtin()
            .ok_or_else(|| format_err!("Only support builtin network"))?,
    );
    let expiration_time = expiration_time + node_info.now;
    let raw_txn = RawUserTransaction::new(
        account.address,
        account_resource.sequence_number(),
        payload,
        max_gas_amount,
        gas_price,
        expiration_time,
        net.chain_id(),
    );

    client.account_sign_txn(raw_txn)
}
