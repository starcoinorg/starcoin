// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use serde::{Serialize, Serializer};
use starcoin_rpc_api::types::{ListCodeView, ListResourceView};
use starcoin_vm_types::account_address::AccountAddress;
use structopt::StructOpt;

/// List state data command
///  Some examples:
///  ``` shell
///  state list code 0x1
///  state list resource 0x1
///  ```
#[derive(Debug, StructOpt)]
#[structopt(name = "list")]
pub enum ListDataOpt {
    Code {
        #[structopt(help = "account address")]
        address: AccountAddress,

        #[structopt(long, short = "n")]
        /// Get state at a special block height.
        block_number: Option<u64>,
    },
    Resource {
        #[structopt(help = "account address")]
        address: AccountAddress,

        #[structopt(long, short = "n")]
        /// Get state at a special block height.
        block_number: Option<u64>,
    },
}

pub struct ListCmd;

pub enum ListDataResult {
    Code(ListCodeView),
    Resource(ListResourceView),
}

impl Serialize for ListDataResult {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Code(c) => c.serialize(serializer),
            Self::Resource(r) => r.serialize(serializer),
        }
    }
}

impl CommandAction for ListCmd {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ListDataOpt;
    type ReturnItem = ListDataResult;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let result = match opt {
            ListDataOpt::Code {
                address,
                block_number,
            } => {
                let state_root = match block_number {
                    Some(block_number) => ctx
                        .state()
                        .client()
                        .chain_get_block_by_number(*block_number)?
                        .map(|block_view| block_view.header.state_root),
                    None => None,
                };
                ListDataResult::Code(
                    ctx.state()
                        .client()
                        .state_list_code(*address, true, state_root)?,
                )
            }
            ListDataOpt::Resource {
                address,
                block_number,
            } => {
                let state_root = match block_number {
                    Some(block_number) => ctx
                        .state()
                        .client()
                        .chain_get_block_by_number(*block_number)?
                        .map(|block_view| block_view.header.state_root),
                    None => None,
                };
                ListDataResult::Resource(
                    ctx.state()
                        .client()
                        .state_list_resource(*address, true, state_root)?,
                )
            }
        };
        Ok(result)
    }
}
