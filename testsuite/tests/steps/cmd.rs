// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use scmd::{CmdContext, Command};
use serde_json::Value;
use starcoin_cmd::dev::GetCoinCommand;
use starcoin_cmd::node::{InfoCommand, PeersCommand};
use starcoin_cmd::view::{AccountWithStateView, NodeInfoView, PeerInfoView, TransactionView};
use starcoin_cmd::wallet::{CreateCommand, ListCommand, ShowCommand, UnlockCommand};
use starcoin_cmd::{wallet, CliState, StarcoinOpt};
use starcoin_logger::prelude::*;
use starcoin_wallet_api::WalletAccount;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .then("[cmd] node info", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let node_info = client.clone().node_info().unwrap();
            let state = CliState::new(node_info.net, client.clone(), None);
            let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
            // let context = world.context.as_mut().take().unwrap( );
            let result = context
                .command(Command::with_name("node").subcommand(InfoCommand))
                .exec_with_args::<NodeInfoView>(vec!["starcoin", "node", "info"])
                .unwrap();
            info!("result:{:?}", result);
        })
        .then("[cmd] node peers", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let node_info = client.clone().node_info().unwrap();
            let state = CliState::new(node_info.net, client.clone(), None);
            let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
            // let context = world.context.as_mut().take().unwrap( );
            let result = context
                .command(Command::with_name("node").subcommand(PeersCommand))
                .exec_with_args::<PeerInfoView>(vec!["starcoin", "node", "peers"])
                .unwrap();
            info!("result:{:?}", result);
        })
        .then("[cmd] wallet list", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let node_info = client.clone().node_info().unwrap();
            let state = CliState::new(node_info.net, client.clone(), None);
            // let state = world.cli_state.take().unwrap();
            let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
            let mut list_result = context
                .command(Command::with_name("wallet").subcommand(ListCommand))
                .exec_with_args::<Vec<WalletAccount>>(vec!["starcoin", "wallet", "list"])
                .unwrap();
            info!("wallet list result:{:?}", list_result);
            world.default_address = Some(list_result.pop().unwrap().address);
        })
        .then("[cmd] wallet show", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let node_info = client.clone().node_info().unwrap();
            let state = CliState::new(node_info.net, client.clone(), None);
            let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
            let show_result = context
                .command(Command::with_name("wallet").subcommand(ShowCommand))
                .exec_with_args::<AccountWithStateView>(vec!["starcoin", "wallet", "show"])
                .unwrap();
            info!("wallet show result:{:?}", show_result);
        })
        .then_regex(
            r#"dev get_coin "([^"]*)""#,
            |world: &mut MyWorld, args, _step| {
                let amount = args[1].as_str();
                let client = world.rpc_client.as_ref().take().unwrap();
                let node_info = client.clone().node_info().unwrap();
                let state = CliState::new(node_info.net, client.clone(), None);
                let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
                let get_result = context
                    .command(Command::with_name("dev").subcommand(GetCoinCommand))
                    .exec_with_args::<TransactionView>(vec![
                        "starcoin", "dev", "get_coin", "-v", amount,
                    ])
                    .unwrap();
                info!("get coin result:{:?}", get_result);
            },
        )
        .then_regex(
            r#"wallet create "([^"]*)""#,
            |world: &mut MyWorld, args, _step| {
                let password = args[1].as_str();
                let client = world.rpc_client.as_ref().take().unwrap();
                let node_info = client.clone().node_info().unwrap();
                let state = CliState::new(node_info.net, client.clone(), None);
                let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
                let create_result = context
                    .command(Command::with_name("wallet").subcommand(CreateCommand))
                    .exec_with_args::<WalletAccount>(vec![
                        "starcoin", "wallet", "create", "-p", password,
                    ])
                    .unwrap();
                world.txn_account = Some(create_result.clone());
                info!("wallet create result:{:?}", create_result);
            },
        )
        .then_regex(
            r#"wallet unlock password:"([^"]*)""#,
            |world: &mut MyWorld, args, _step| {
                let password = args[1].as_str();
                let client = world.rpc_client.as_ref().take().unwrap();
                let node_info = client.clone().node_info().unwrap();
                let state = CliState::new(node_info.net, client.clone(), None);
                let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
                let unlock_result = context
                    .command(Command::with_name("wallet").subcommand(UnlockCommand))
                    .exec_with_args::<String>(vec![
                        "starcoin",
                        "wallet",
                        "unlock",
                        "account_address",
                        "-p",
                        password,
                    ])
                    .unwrap();
                info!("wallet unlock result:{:?}", unlock_result);
            },
        )
        .then_regex(
            r#"cmd cli: "([^"]*)""#,
            |world: &mut MyWorld, args, _step| {
                let client = world.rpc_client.as_ref().take().unwrap();
                let node_info = client.clone().node_info().unwrap();
                let state = CliState::new(node_info.net, client.clone(), None);
                let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
                // world.context = Some(context);
                let mut vec = vec![];
                vec.push("starcoin");
                for parameter in args[1].as_str().split_whitespace() {
                    vec.push(parameter);
                }
                let result = context
                    .command(
                        Command::with_name("wallet")
                            .subcommand(wallet::CreateCommand)
                            .subcommand(wallet::ShowCommand),
                    )
                    .exec_with_args::<Value>(vec)
                    .unwrap();
                println!("cmd cli: {:?}", result);
                info!("cmd cli: {:?}", result);
            },
        );
    builder.build()
}
