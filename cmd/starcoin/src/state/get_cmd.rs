// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use serde::{Serialize, Serializer};
use starcoin_rpc_api::types::{CodeView, ResourceView, StrView};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::language_storage::{ModuleId, StructTag};
use structopt::StructOpt;

/// Get contract data command
///  Some examples:
///  ``` shell
///  state get code 0x1::Account
///  state get resource 0x1 0x1::Account::Account
///  ```
#[derive(Debug, StructOpt)]
#[structopt(name = "get")]
pub enum GetOpt {
    Code {
        #[structopt(help = "module id like: 0x1::Account")]
        module_id: StrView<ModuleId>,
    },
    Resource {
        #[structopt(help = "account address")]
        address: AccountAddress,
        #[structopt(help = "resource struct tag,", default_value = "0x1::Account::Account")]
        resource_type: StrView<StructTag>,
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
            GetOpt::Code { module_id } => GetDataResult::Code(
                ctx.state()
                    .client()
                    .state_get_code(module_id.0.clone(), true)?,
            ),
            GetOpt::Resource {
                address,
                resource_type,
            } => GetDataResult::Resource(ctx.state().client().state_get_resource(
                *address,
                resource_type.0.clone(),
                true,
            )?),
        };

        Ok(result)
    }
}
