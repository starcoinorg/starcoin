// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cli_state::CliState, view_vm2::AccountWithStateView, StarcoinOpt};
use anyhow::{format_err, Result};
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_client::StateRootOption;
use starcoin_vm2_crypto::ValidCryptoMaterialStringExt;
use starcoin_vm2_statedb::ChainStateReader;
use starcoin_vm2_vm_types::{account_address::AccountAddress, state_view::StateReaderExt};
use std::collections::HashMap;

/// Show a account info, only the accounts managed by the current node are supported
#[derive(Debug, Parser, Default)]
#[clap(name = "show")]
pub struct ShowOpt {
    #[clap(name = "address_or_receipt")]
    /// The account's address to show, if absent, show the default account.
    address_or_receipt: Option<AccountAddress>,

    //`b` and `block_id` for compat with previous cli option.
    #[clap(name = "state-root", long, short = 'b', alias = "block_id")]
    /// The block number or block hash for get state, if absent, use latest block state_root.
    state_root: Option<StateRootOption>,
}

pub struct ShowCommand;

impl CommandAction for ShowCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ShowOpt;
    type ReturnItem = AccountWithStateView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let rpc_client = ctx.state().client();
        let account_client = ctx.state().vm2()?.account_client();
        let opt = ctx.opt();
        let account_address = if let Some(address_or_receipt) = opt.address_or_receipt {
            address_or_receipt
        } else {
            let default_account = account_client
                .get_default_account()?
                .ok_or_else(|| format_err!("Default account should exist."))?;
            default_account.address
        };
        let account = account_client
            .get_account(account_address)?
            .ok_or_else(|| {
                format_err!("Account with address {} does not exist.", account_address)
            })?;

        let chain_state_reader = rpc_client.state_reader2(opt.state_root.unwrap_or_default())?;
        let sequence_number = chain_state_reader
            .get_account_resource(*account.address())
            .map(|r| r.sequence_number())
            .ok();

        let _resources = rpc_client.state_list_resource2(
            *account.address(),
            false,
            Some(chain_state_reader.state_root()),
            0,
            usize::MAX,
            None,
        )?;
        // TODO(BobOng): [dual-vm] decode function not valid now
        // let balances: HashMap<TokenCode, u128> = resources
        //     .resources
        //     .into_iter()
        //     .filter_map(|(resource_type, resource)| {
        //         if let Some(token_code) = BalanceResource::token_code(&resource_type.0) {
        //             let balance = resource
        //                 .decode::<BalanceResource>()
        //                 .ok()
        //                 .map(|balance| balance.token());
        //             Some((token_code, balance.unwrap_or(0)))
        //         } else {
        //             None
        //         }
        //     })
        //     .collect();

        let balances = HashMap::new();

        let auth_key = account.public_key.authentication_key();
        Ok(AccountWithStateView {
            auth_key: auth_key.to_encoded_string()?,
            account,
            sequence_number,
            balances,
        })
    }
}
