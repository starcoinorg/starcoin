// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use starcoin_account_api::AccountInfo;
use starcoin_crypto::HashValue;
use starcoin_executor::DEFAULT_EXPIRATION_TIME;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::{RpcClient, StateRootOption};
use starcoin_state_api::StateReaderExt;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_value::TokenValue;
use starcoin_types::account_config::STCUnit;
use std::time::Duration;

pub struct Faucet {
    client: RpcClient,
    faucet_account: AccountInfo,
    faucet_account_password: String,
    max_amount_pre_request: TokenValue<STCUnit>,
}

const DEFAULT_GAS_PRICE: u64 = 1;
const MAX_GAS: u64 = 1000000;

impl Faucet {
    pub fn new(
        client: RpcClient,
        faucet_account: AccountInfo,
        faucet_account_password: String,
        max_amount_pre_request: TokenValue<STCUnit>,
    ) -> Self {
        Faucet {
            client,
            faucet_account,
            faucet_account_password,
            max_amount_pre_request,
        }
    }

    pub fn transfer(
        &self,
        amount: Option<TokenValue<STCUnit>>,
        receiver: AccountAddress,
    ) -> Result<HashValue> {
        let amount = amount
            .and_then(|value| {
                if value.scaling() > self.max_amount_pre_request.scaling() {
                    None
                } else {
                    Some(value)
                }
            })
            .unwrap_or(self.max_amount_pre_request);

        let sequence_number = match self
            .client
            .next_sequence_number_in_txpool(*self.faucet_account.address())?
        {
            Some(sequence_number) => sequence_number,
            None => {
                let chain_state_reader = self.client.state_reader(StateRootOption::Latest)?;
                chain_state_reader
                    .get_account_resource(*self.faucet_account.address())?
                    .ok_or_else(|| {
                        format_err!(
                            "Can not find account on chain by address:{}",
                            self.faucet_account.address()
                        )
                    })?
                    .sequence_number()
            }
        };
        let node_info = self.client.node_info()?;
        let raw_tx = starcoin_executor::build_transfer_txn(
            self.faucet_account.address,
            receiver,
            sequence_number,
            amount.scaling(),
            DEFAULT_GAS_PRICE,
            MAX_GAS,
            node_info.now_seconds + DEFAULT_EXPIRATION_TIME,
            node_info.net.chain_id(),
        );
        info!("sender transaction: {:?}", raw_tx);
        self.client.account_unlock(
            self.faucet_account.address,
            self.faucet_account_password.clone(),
            Duration::from_secs(30),
        )?;
        let signed_tx = self.client.account_sign_txn(raw_tx)?;
        //ignore lock result
        let _res = self.client.account_lock(self.faucet_account.address);
        self.client.submit_transaction(signed_tx)
    }
}
