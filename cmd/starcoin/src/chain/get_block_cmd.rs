// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::GetBlockOption;
use starcoin_rpc_api::types::BlockView;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum HashOrNumber {
    Hash(HashValue),
    Number(u64),
}

/// Get block by hash or number.
#[derive(Debug, Parser)]
#[clap(name = "get-block", alias = "get_block")]
pub struct GetBlockOpt {
    #[clap(name = "hash-or-number")]
    hash_or_number: HashOrNumber,

    #[arg(name = "contains-raw-block", default_value_t = false)]
    raw: std::primitive::bool,
}

impl FromStr for HashOrNumber {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match u64::from_str(s) {
            Ok(number) => Ok(Self::Number(number)),
            Err(_) => Ok(Self::Hash(HashValue::from_str(s)?)),
        }
    }
}

pub struct GetBlockCommand;

impl CommandAction for GetBlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetBlockOpt;
    type ReturnItem = BlockView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let block = match opt.hash_or_number {
            HashOrNumber::Hash(hash) => client
                .chain_get_block_by_hash(
                    hash,
                    Some(GetBlockOption {
                        decode: true,
                        raw: opt.raw,
                    }),
                )?
                .ok_or_else(|| anyhow::format_err!("block of hash {} not found", hash))?,
            HashOrNumber::Number(number) => client
                .chain_get_block_by_number(
                    number,
                    Some(GetBlockOption {
                        decode: true,
                        raw: opt.raw,
                    }),
                )?
                .ok_or_else(|| anyhow::format_err!("block of height {} not found", number))?,
        };
        Ok(block)
    }
}
