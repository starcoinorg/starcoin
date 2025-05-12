// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use jpst::TemplateContext;
use scmd::{result_to_json, CmdContext};
use serde_json::Value;
use starcoin_account_provider::ProviderFactory;
use starcoin_cmd::add_command;
use starcoin_cmd::{CliState, StarcoinOpt};
use starcoin_config::account_provider_config::AccountProviderConfig;
use starcoin_config::{G_APP_VERSION, G_CRATE_VERSION};
use starcoin_logger::prelude::*;
use std::sync::Arc;
use std::time::Duration;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .then_regex(
            r#"cmd: "(([^"\\]|\\).*)""#,
            |world: &mut MyWorld, args, _step| {
                let client = world.default_rpc_client.as_ref().take().unwrap();
                let chain_id = client.node_info().unwrap().net.chain_id();
                let account_client = ProviderFactory::create_provider(
                    client.clone(),
                    chain_id,
                    &AccountProviderConfig::default(),
                )
                .unwrap();
                let node_info = client.clone().node_info().unwrap();
                let state = CliState::new(
                    node_info.net,
                    client.clone(),
                    Some(Duration::from_secs(5)),
                    None,
                    Arc::new(account_client),
                    None,
                );
                let context = CmdContext::<CliState, StarcoinOpt>::with_state(
                    G_CRATE_VERSION,
                    Some(G_APP_VERSION.as_str()),
                    state,
                );
                if world.tpl_ctx.is_none() {
                    world.tpl_ctx = Some(TemplateContext::new());
                }
                let tpl_ctx = world.tpl_ctx.as_mut().unwrap();
                // get last cmd result as current parameter
                let mut vec = vec!["starcoin"];

                let evaled_parameters = eval_command_args(tpl_ctx, args[1].parse().unwrap());
                let parameters = evaled_parameters.split_whitespace();

                for parameter in parameters {
                    vec.push(parameter);
                }

                let cmd = vec.get(1).cloned().unwrap();

                let result = add_command(context).exec_with_args::<Value>(vec);

                //TODO support error and check error in the cmd.feature
                if result.is_err() {
                    panic!("{}", result.unwrap_err());
                }
                let result_json = result_to_json(&result);

                debug!("cmd {} execute result: {:?}", cmd, result_json);
                tpl_ctx.entry(cmd).append(result_json);
            },
        )
        .then_regex(
            r#"assert: "([^"]*)""#,
            |world: &mut MyWorld, args, _step| {
                let evaled_parameters =
                    eval_command_args(world.tpl_ctx.as_ref().unwrap(), args[1].to_owned());
                let parameters = evaled_parameters.split_whitespace().collect::<Vec<_>>();

                for chunk in parameters.chunks(3) {
                    let first = chunk.get(0).cloned();
                    let op = chunk.get(1).cloned();
                    let second = chunk.get(2).cloned();

                    info!("assert value: {:?} {:?} {:?}", first, op, second);

                    match (first, op, second) {
                        (Some(first), Some(op), Some(second)) => match op {
                            "==" => assert_eq!(first, second),
                            "!=" => assert_ne!(first, second),
                            _ => panic!("unsupported operator"),
                        },
                        _ => panic!("expected 3 arguments: first [==|!=] second"),
                    }
                }
                info!("assert ok!");
            },
        );
    builder.build()
}

fn eval_command_args(ctx: &TemplateContext, args: String) -> String {
    info!("args: {}", args);
    let args = args.replace("\\\"", "\"");
    let eval_args = jpst::format_str!(&args, ctx);
    info!("eval args:{}", eval_args);
    eval_args
}
