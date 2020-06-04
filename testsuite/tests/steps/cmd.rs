// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use scmd::{CommandAction, ExecContext};
use starcoin_cmd::node::{InfoCommand, InfoOpt};
use starcoin_logger::prelude::*;
use std::sync::Arc;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder.given("[cmd] node info", |world: &mut MyWorld, _step| {
        let mut context = world.context.take().unwrap();
        let info_cmd = InfoCommand::new();
        let (global_opt, state) = context.matcher_and_opt(vec![]).unwrap();
        let exe_context = ExecContext::new(
            Arc::new(state),
            Arc::new(global_opt),
            Arc::new(InfoOpt::new()),
        );

        let result = info_cmd.run(&exe_context).unwrap();
        info!("result:{:?}", result);
    });
    builder.build()
}
