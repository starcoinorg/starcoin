// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use scmd::{CommandAction, ExecContext};
use starcoin_cmd::dev::{GetCoinCommand, GetCoinOpt};
use starcoin_cmd::node::{InfoCommand, InfoOpt};
use starcoin_cmd::wallet::{
    CreateCommand, CreateOpt, ListCommand, ListOpt, ShowCommand, ShowOpt, UnlockCommand, UnlockOpt,
};
use starcoin_logger::prelude::*;
use std::sync::Arc;
use structopt::StructOpt;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .then("[cmd] node info", |world: &mut MyWorld, _step| {
            let context = world.context.as_mut().unwrap();
            let (global_opt, state) = context.matcher_and_opt(vec![]).unwrap();
            let exe_context = ExecContext::new(
                Arc::new(state),
                Arc::new(global_opt),
                Arc::new(InfoOpt::default()),
            );
            let result = InfoCommand.run(&exe_context).unwrap();
            info!("result:{:?}", result);
        })
        .then("[cmd] wallet list", |world: &mut MyWorld, _step| {
            let context = world.context.as_mut().unwrap();
            let (global_opt, state) = context.matcher_and_opt(vec![]).unwrap();
            let exe_context = ExecContext::new(
                Arc::new(state),
                Arc::new(global_opt),
                Arc::new(ListOpt::default()),
            );
            let mut list_result = ListCommand.run(&exe_context).unwrap();
            info!("wallet list result:{:?}", list_result);
            world.default_address = Some(list_result.pop().unwrap().address);
        })
        .then("[cmd] wallet show", |world: &mut MyWorld, _step| {
            let context = world.context.as_mut().unwrap();
            let address = world.default_address.as_ref().take().unwrap();
            let (global_opt, state) = context.matcher_and_opt(vec![]).unwrap();
            let show_opt =
                ShowOpt::from_iter(vec!["account_address", address.to_string().as_str()]);
            let exe_context =
                ExecContext::new(Arc::new(state), Arc::new(global_opt), Arc::new(show_opt));
            let show_result = ShowCommand.run(&exe_context).unwrap();
            info!("wallet show result:{:?}", show_result);
        })
        .then_regex(
            r#"dev get_coin "([^"]*)""#,
            |world: &mut MyWorld, args, _step| {
                let amount = args[1].as_str();
                let context = world.context.as_mut().unwrap();
                let (global_opt, state) = context.matcher_and_opt(vec![]).unwrap();
                let get_coin_opt = GetCoinOpt::from_iter(vec!["", "-v", amount]);
                let exe_context = ExecContext::new(
                    Arc::new(state),
                    Arc::new(global_opt),
                    Arc::new(get_coin_opt),
                );
                let get_result = GetCoinCommand.run(&exe_context).unwrap();
                info!("dev get_coin result:{:?}", get_result);
            },
        )
        .then_regex(
            r#"wallet create "([^"]*)""#,
            |world: &mut MyWorld, args, _step| {
                let password = args[1].as_str();
                let context = world.context.as_mut().unwrap();
                let (global_opt, state) = context.matcher_and_opt(vec![]).unwrap();
                let create_opt = CreateOpt::from_iter(vec!["", "-p", password]);
                let exe_context =
                    ExecContext::new(Arc::new(state), Arc::new(global_opt), Arc::new(create_opt));
                let create_result = CreateCommand.run(&exe_context).unwrap();
                world.txn_account = Some(create_result.clone());
                info!("wallet create result:{:?}", create_result);
            },
        )
        .then_regex(
            r#"wallet unlock password:"([^"]*)""#,
            |world: &mut MyWorld, args, _step| {
                let password = args[1].as_str();
                let context = world.context.as_mut().unwrap();
                let txn_account = world.txn_account.as_ref().take().unwrap();
                let (global_opt, state) = context.matcher_and_opt(vec![]).unwrap();
                let unlock_opt = UnlockOpt::from_iter(vec![
                    "",
                    txn_account.address.to_string().as_str(),
                    "-p",
                    password,
                ]);
                let exe_context =
                    ExecContext::new(Arc::new(state), Arc::new(global_opt), Arc::new(unlock_opt));
                let unlock_result = UnlockCommand.run(&exe_context).unwrap();
                info!("wallet unlock result:{:?}", unlock_result);
            },
        );
    builder.build()
}
