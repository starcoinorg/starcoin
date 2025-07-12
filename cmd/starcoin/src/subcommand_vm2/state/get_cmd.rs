// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use serde::{Serialize, Serializer};
use starcoin_vm2_types::view::{CodeView, ResourceView, StrView};
use starcoin_vm2_vm_types::account_address::AccountAddress;
use starcoin_vm2_vm_types::language_storage::{ModuleId, StructTag};

/// Get state data command
///  Some examples:
///  ``` shell
///  state get code 0x1::Account
///  state get resource 0x1 0x1::Account::Account
///  ```
#[derive(Debug, Parser)]
#[clap(name = "get")]
pub enum GetOpt {
    Code {
        #[clap(help = "module id like: 0x1::Account")]
        module_id: StrView<ModuleId>,
        #[clap(long, short = 'n')]
        /// Get state at a special block height.
        block_number: Option<u64>,
    },
    Resource {
        #[clap(help = "account address")]
        address: AccountAddress,
        #[clap(help = "resource struct tag,", default_value = "0x1::Account::Account")]
        resource_type: StrView<StructTag>,
        #[clap(long, short = 'n')]
        /// Get state at a special block height.
        block_number: Option<u64>,
    },
}

pub struct GetCommand;

pub enum GetDataResult {
    Code(Option<CodeView>),
    Resource(Option<ResourceView>),
}

impl Serialize for GetDataResult {
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

impl CommandAction for GetCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetOpt;
    type ReturnItem = GetDataResult;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let result = match opt {
            GetOpt::Code {
                module_id,
                block_number,
            } => {
                let state_root = match block_number {
                    Some(block_number) => ctx
                        .state()
                        .client()
                        .chain_get_block_by_number(*block_number, None)?
                        .map(|block_view| block_view.header.state_root),
                    None => None,
                };
                GetDataResult::Code(ctx.state().client().state_get_code2(
                    module_id.0.clone(),
                    true,
                    state_root,
                )?)
            }
            GetOpt::Resource {
                address,
                resource_type,
                block_number,
            } => {
                let state_root = match block_number {
                    Some(block_number) => ctx
                        .state()
                        .client()
                        .chain_get_block_by_number(*block_number, None)?
                        .map(|block_view| block_view.header.state_root),
                    None => None,
                };
                GetDataResult::Resource(ctx.state().client().state_get_resource2(
                    *address,
                    resource_type.0.clone(),
                    true,
                    state_root,
                )?)
            }
        };

        Ok(result)
    }
}
