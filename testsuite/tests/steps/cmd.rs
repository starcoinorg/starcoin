// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use jsonpath::Selector;
use scmd::CmdContext;
use serde_json::Value;
use starcoin_cmd::add_command;
use starcoin_cmd::view::{AccountWithStateView, NodeInfoView, PeerInfoView, TransactionView};
use starcoin_cmd::{CliState, StarcoinOpt};
use starcoin_logger::prelude::*;
use starcoin_wallet_api::WalletAccount;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .then("[cmd] node info", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let node_info = client.clone().node_info().unwrap();
            let state = CliState::new(node_info.net, client.clone(), None, None);
            let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
            // let context = world.context.as_mut().take().unwrap( );
            let result = add_command(context)
                .exec_with_args::<NodeInfoView>(vec!["starcoin", "node", "info"])
                .unwrap();
            info!("result:{:?}", result);
        })
        .then("[cmd] node peers", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let node_info = client.clone().node_info().unwrap();
            let state = CliState::new(node_info.net, client.clone(), None, None);
            let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
            // let context = world.context.as_mut().take().unwrap( );
            let result = add_command(context)
                .exec_with_args::<PeerInfoView>(vec!["starcoin", "node", "peers"])
                .unwrap();
            info!("result:{:?}", result);
        })
        .then("[cmd] wallet list", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let node_info = client.clone().node_info().unwrap();
            let state = CliState::new(node_info.net, client.clone(), None, None);
            // let state = world.cli_state.take().unwrap();
            let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
            let mut list_result = add_command(context)
                .exec_with_args::<Vec<WalletAccount>>(vec!["starcoin", "wallet", "list"])
                .unwrap();
            info!("wallet list result:{:?}", list_result);
            world.default_address = Some(list_result.pop().unwrap().address);
        })
        .then("[cmd] wallet show", |world: &mut MyWorld, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();
            let node_info = client.clone().node_info().unwrap();
            let state = CliState::new(node_info.net, client.clone(), None, None);
            let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
            let show_result = add_command(context)
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
                let state = CliState::new(node_info.net, client.clone(), None, None);
                let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
                let inter_args = if amount.eq("<amount>") {
                    vec!["starcoin", "dev", "get_coin"]
                } else {
                    vec!["starcoin", "dev", "get_coin", "-v", amount]
                };
                println!("inter {:?}", inter_args);
                let get_result = add_command(context)
                    .exec_with_args::<TransactionView>(inter_args)
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
                let state = CliState::new(node_info.net, client.clone(), None, None);
                let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
                let create_result = add_command(context)
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
                let state = CliState::new(node_info.net, client.clone(), None, None);
                let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
                let unlock_result = add_command(context)
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
        .then_regex(r#"cmd: "([^"]*)""#, |world: &mut MyWorld, args, _step| {
            let client = world.rpc_client.as_ref().take().unwrap();

            let node_info = client.clone().node_info().unwrap();
            let state = CliState::new(node_info.net, client.clone(), None, None);
            let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
            // get last cmd result as current parameter
            let mut vec = vec!["starcoin"];
            let mut rex_parameter = "";
            for parameter in args[1].as_str().split_whitespace() {
                if !parameter.starts_with("$") {
                    vec.push(parameter);
                }
                rex_parameter = parameter;
            }

            if world.cmd_value.as_ref().is_some() {
                for parameter in world.cmd_value.as_ref().take().unwrap() {
                    vec.push(parameter.as_str());
                }
            };
            info!("parameter: {:?}", vec.clone());
            let result = add_command(context).exec_with_args::<Value>(vec).unwrap();
            info!("cmd rex_parameter: {:?}", rex_parameter);
            // parse result
            let selector = Selector::new(rex_parameter).unwrap();
            let next_value: Vec<String> = selector
                .find(&result)
                .map(|t| t.as_str().unwrap().to_string())
                .collect();
            info!("next value: {:?}", next_value.clone());
            world.cmd_value = Some(next_value);
            info!("cmd continuous: {:?}", result);
        })
        .then_regex(
            r#"cmd cli: "([^"]*)""#,
            |world: &mut MyWorld, args, _step| {
                let client = world.rpc_client.as_ref().take().unwrap();
                let node_info = client.clone().node_info().unwrap();
                let state = CliState::new(node_info.net, client.clone(), None, None);
                let context = CmdContext::<CliState, StarcoinOpt>::with_state(state);
                // world.context = Some(context);
                let mut vec = vec![];
                vec.push("starcoin");
                for parameter in args[1].as_str().split_whitespace() {
                    vec.push(parameter);
                }
                let result = add_command(context).exec_with_args::<Value>(vec).unwrap();
                println!("cmd cli: {:?}", result);
                info!("cmd cli: {:?}", result);
            },
        );
    builder.build()
}
