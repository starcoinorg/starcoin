// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_logger::prelude::*;
use structopt::StructOpt;

/// Verify node sync.
#[derive(Debug, StructOpt)]
#[structopt(name = "node")]
pub struct NodeOpt {
    #[structopt(name = "rate", long, short = "r", default_value = "0.85")]
    rate: f32,
}

fn mean(data: &[u64]) -> Option<f32> {
    let sum = data.iter().sum::<u64>() as f32;
    let count = data.len();

    match count {
        positive if positive > 0 => Some(sum / count as f32),
        _ => None,
    }
}

fn std_deviation(data: &[u64]) -> Option<f32> {
    match (mean(data), data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = data
                .iter()
                .map(|value| {
                    let diff = data_mean - (*value as f32);
                    diff * diff
                })
                .sum::<f32>()
                / count as f32;

            Some(variance.sqrt())
        }
        _ => None,
    }
}

fn score_rate(data: &[u64], data_mean: Option<f32>, data_std_deviation: Option<f32>) -> f32 {
    if data.is_empty() {
        return 1f32;
    }
    let mut negative_count = 0;
    for data_i in data.to_vec() {
        let zscore = match (data_mean, data_std_deviation) {
            (Some(mean), Some(std_deviation)) => {
                let diff = data_i as f32 - mean;

                Some(diff / std_deviation)
            }
            _ => None,
        };
        if zscore.unwrap() < 0 as f32 {
            negative_count += 1;
        }
    }
    let size = data.len() as f32;
    1f32 - negative_count as f32 / size
}

pub struct VerifyNodeCommand;

impl CommandAction for VerifyNodeCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = NodeOpt;
    type ReturnItem = String;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let rate = opt.rate;

        let node_peers = client.node_peers()?;
        let mut head_number_vec = vec![];
        for peer in node_peers {
            head_number_vec.push(peer.chain_info.head.number.0);
        }
        if head_number_vec.is_empty() {
            warn!("peers is empty!");
            Ok("peers is empty!".parse()?)
        } else {
            let mean = mean(head_number_vec.as_slice());
            let std_deviation = std_deviation(head_number_vec.as_slice());
            let score_rate = score_rate(head_number_vec.as_slice(), mean, std_deviation);
            if score_rate > rate {
                Ok("verify ok!".parse()?)
            } else {
                Ok(format!("score_rate: {}", score_rate))
            }
        }
    }
}
