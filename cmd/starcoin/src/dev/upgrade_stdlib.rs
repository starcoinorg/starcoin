// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use anyhow::{bail, format_err, Result};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::transaction::RawUserTransaction;
use starcoin_vm_types::genesis_config::ChainNetwork;
use starcoin_vm_types::transaction::{SignedUserTransaction, TransactionPayload};

pub fn to_txn_with_association_account_by_rpc_client(
    cli_state: &CliState,
    max_gas_amount: u64,
    gas_price: u64,
    expiration_time: u64,
    payload: TransactionPayload,
) -> Result<SignedUserTransaction> {
    let association_account = cli_state.association_account()?;
    let client = cli_state.client();
    let node_info = client.node_info()?;
    if let Some(association_account) = association_account {
        let chain_state_reader = RemoteStateReader::new(client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader
            .get_account_resource(association_account.address())?
            .ok_or_else(|| format_err!("association_account must exist on chain."))?;
        let net = ChainNetwork::new_builtin(
            *cli_state
                .net()
                .as_builtin()
                .ok_or_else(|| format_err!("Only support builtin network"))?,
        );
        let expiration_time = expiration_time + node_info.now;
        let raw_txn = RawUserTransaction::new(
            association_account.address,
            account_resource.sequence_number(),
            payload,
            max_gas_amount,
            gas_price,
            expiration_time,
            net.chain_id(),
        );

        client.account_sign_txn(raw_txn)
    } else {
        bail!("association_account not exists in wallet, please import association_account first.",);
    }
}
