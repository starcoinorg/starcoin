// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use jsonpath::Selector;
use lazy_static::lazy_static;
use regex::Regex;
use scmd::CmdContext;
use serde_json::Value;
use starcoin_account_provider::ProviderFactory;
use starcoin_cmd::add_command;
use starcoin_cmd::{CliState, StarcoinOpt};
use starcoin_config::account_provider_config::AccountProviderConfig;
use starcoin_config::{APP_VERSION, CRATE_VERSION};
use starcoin_logger::prelude::*;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

lazy_static! {
    static ref SELECTOR_REGEX: Regex = Regex::new(r"@(?P<value>[^@\s]+)@").unwrap();
}
fn extract_selector_str(input: &str) -> HashSet<&str> {
    SELECTOR_REGEX
        .captures_iter(input)
        .filter_map(|cap| cap.name("value").map(|find| find.as_str()))
        .collect()
}

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder
        .then_regex(r#"cmd: "([^"]*)""#, |world: &mut MyWorld, args, _step| {
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
                account_client,
            );
            let context = CmdContext::<CliState, StarcoinOpt>::with_state(
                CRATE_VERSION,
                Some(APP_VERSION.as_str()),
                state,
            );
            // get last cmd result as current parameter
            let mut vec = vec!["starcoin"];
            let parameters = get_command_args(world, (*args[1]).parse().unwrap());
            for parameter in parameters.as_str().split_whitespace() {
                vec.push(parameter);
            }
            info!("parameter: {:?}", vec.clone());
            let result = add_command(context).exec_with_args::<Value>(vec).unwrap();

            debug!("cmd execute result: {:?}", result);

            world.value = Some(result);
        })
        .then_regex(
            r#"assert: "([^"]*)""#,
            |world: &mut MyWorld, args, _step| {
                //get field_name from args
                let mut args_vec = vec![];
                for parameter in args[1].as_str().split_whitespace() {
                    args_vec.push(parameter);
                }
                let mut args_map = HashMap::new();
                for (i, v) in args_vec.iter().enumerate() {
                    if (i % 2) == 1 {
                        continue;
                    }
                    if i + 1 <= args_vec.len() {
                        args_map.insert(*v, args_vec.get(i + 1).unwrap());
                    }
                }
                for (arg_key, arg_val) in args_map {
                    if let Some(value) = &world.value {
                        let selector = Selector::new(arg_key).unwrap();
                        let mut value: Vec<String> = selector
                            .find(&value)
                            .map(|t| {
                                if t.is_string() {
                                    t.as_str().unwrap().to_string()
                                } else {
                                    t.to_string()
                                }
                            })
                            .collect();
                        info!("assert value: {:?},expect: {:?}", value, *arg_val);
                        assert_eq!(value.pop().unwrap().as_str(), *arg_val);
                    }
                }
                info!("assert ok!");
            },
        );
    builder.build()
}

fn get_command_args(world: &mut MyWorld, args: String) -> String {
    let args_set = extract_selector_str(args.as_str());
    info!("extract str: {:?}", args_set.clone());
    let mut replace_map = HashMap::new();
    for key in args_set {
        if let Some(value) = &world.value {
            let selector = Selector::new(key).unwrap();
            let mut next_value: Vec<&str> =
                selector.find(&value).map(|t| t.as_str().unwrap()).collect();
            if !next_value.is_empty() {
                replace_map.insert(key, next_value.pop().unwrap());
            }
        }
    }
    info!("replace_map: {:?}", replace_map.clone());
    //replace args
    let mut result = args.clone();
    for (arg_key, arg_val) in replace_map {
        let key = "@".to_owned() + arg_key + "@";
        if arg_key.ends_with("auth_key") {
            let val = "\"".to_owned() + arg_val + "\"";
            result = result.replace(key.as_str(), val.as_str());
        } else {
            result = result.replace(key.as_str(), arg_val);
        }
    }

    info!("replace result:{:?}", result);
    result
}
