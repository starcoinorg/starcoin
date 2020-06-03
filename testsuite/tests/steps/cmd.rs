// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use clap::ArgMatches;
use cucumber::{Steps, StepsBuilder};
use scmd::{CommandAction, ExecContext};
use starcoin_cmd::node::{InfoCommand, InfoOpt};
use std::sync::Arc;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder.given("[cmd] node info", |world: &mut MyWorld, _step| {
        let context = world.context.as_ref().take().unwrap();
        let info_cmd = InfoCommand::new();
        let matches = ArgMatches::new();
        // println!("context :{:?}", context);
        let (global_opt, state) = context.init_global_opt(&matches).unwrap();
        let execontext = ExecContext::new(
            Arc::new(state),
            Arc::new(global_opt),
            Arc::new(InfoOpt::new()),
        );

        let result = info_cmd.run(&execontext).unwrap();
        println!("result:{:?}", result);
    });
    builder.build()
}
