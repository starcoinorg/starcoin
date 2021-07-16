// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use serde::{Serialize, Serializer};
use starcoin_rpc_api::types::{AnnotatedMoveStructView, StrView};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::language_storage::{ModuleId, StructTag};
use structopt::StructOpt;

/// Get contract data command
/// Note: this command is deprecated, please use `state get` command.
///  Some examples:
///  ``` shell
///  contract get code 0x1::Account
///  contract get resource 0x1 0x1::Account::Account
///  ```
#[derive(Debug, StructOpt)]
#[structopt(name = "get")]
pub enum GetContractDataOpt {
    Code {
        #[structopt(help = "module id like: 0x1::Account")]
        module_id: StrView<ModuleId>,
    },
    Resource {
        #[structopt(help = "account address")]
        address: AccountAddress,
        #[structopt(
            help = "resource struct tag,",
            default_value = "0x1::Account::Balance<0x1::STC::STC>"
        )]
        resource_type: StrView<StructTag>,
    },
}

pub struct GetContractDataCommand;

pub enum GetContractDataResult {
    Code(Option<String>),
    Resource(Option<AnnotatedMoveStructView>),
}
impl Serialize for GetContractDataResult {
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

impl CommandAction for GetContractDataCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetContractDataOpt;
    type ReturnItem = GetContractDataResult;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        eprintln!("`contract get` command is deprecated, please use `state get` command.");
        let opt = ctx.opt();
        let result = match opt {
            GetContractDataOpt::Code { module_id } => {
                GetContractDataResult::Code(ctx.state().client().get_code(module_id.0.clone())?)
            }
            GetContractDataOpt::Resource {
                address,
                resource_type,
            } => GetContractDataResult::Resource(
                ctx.state()
                    .client()
                    .get_resource(*address, resource_type.0.clone())?
                    .map(Into::into),
            ),
        };
        Ok(result)
    }
}
