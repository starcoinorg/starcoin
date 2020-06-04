// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use scmd::{CommandAction, ExecContext};
use starcoin_cmd::dev::{GetCoinCommand, GetCoinOpt};
use starcoin_cmd::node::{InfoCommand, InfoOpt};
use starcoin_cmd::wallet::{CreateCommand, CreateOpt, ListCommand, ListOpt, ShowCommand, ShowOpt};
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
            println!("result:{:?}", result);
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
            println!("wallet list result:{:?}", list_result);
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
            println!("wallet show result:{:?}", show_result);
            info!("wallet show result:{:?}", show_result);
        })
        .then("[cmd] dev get_coin", |world: &mut MyWorld, _step| {
            let context = world.context.as_mut().unwrap();
            let (global_opt, state) = context.matcher_and_opt(vec![]).unwrap();
            let get_coin_opt = GetCoinOpt::from_iter(vec!["", "-v", "2000000"]);
            let exe_context = ExecContext::new(
                Arc::new(state),
                Arc::new(global_opt),
                Arc::new(get_coin_opt),
            );
            let get_result = GetCoinCommand.run(&exe_context).unwrap();
            println!("dev get_coin result:{:?}", get_result);
            info!("dev get_coin result:{:?}", get_result);
        })
        .then("[cmd] wallet create", |world: &mut MyWorld, _step| {
            let context = world.context.as_mut().unwrap();
            let (global_opt, state) = context.matcher_and_opt(vec![]).unwrap();
            let create_opt = CreateOpt::from_iter(vec!["", "-p", "sfsd333"]);
            let exe_context =
                ExecContext::new(Arc::new(state), Arc::new(global_opt), Arc::new(create_opt));
            let create_result = CreateCommand.run(&exe_context).unwrap();
            println!("wallet create result:{:?}", create_result);
            info!("wallet create result:{:?}", create_result);
        });
    builder.build()
}
