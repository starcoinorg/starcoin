// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use serde::{Serialize, Serializer};
use starcoin_abi_resolver::ABIResolver;
use starcoin_rpc_api::types::{ListCodeView, ListResourceView};
use starcoin_rpc_client::StateRootOption;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::state_view::StateView;
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
                        .chain_get_block_by_number(*block_number, None)?
                        .map(|block_view| block_view.header.state_root),
                    None => None,
                };
                let state_reader = ctx.state().client().state_reader(
                    state_root.map_or(StateRootOption::Latest, StateRootOption::BlockHash),
                )?;
                ListDataResult::Code(resolve_code(
                    &state_reader,
                    *address,
                    ctx.state()
                        .client()
                        .state_list_code(*address, false, state_root)?,
                ))
            }
            ListDataOpt::Resource {
                address,
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
                let state_reader = ctx.state().client().state_reader(
                    state_root.map_or(StateRootOption::Latest, StateRootOption::BlockHash),
                )?;
                ListDataResult::Resource(decode_resource(
                    &state_reader,
                    ctx.state()
                        .client()
                        .state_list_resource(*address, false, state_root)?,
                ))
            }
        };
        Ok(result)
    }
}

fn decode_resource(state_view: &dyn StateView, mut list: ListResourceView) -> ListResourceView {
    list.resources
        .iter_mut()
        .for_each(|(tag_view, resource_view)| {
            match starcoin_dev::playground::view_resource(
                state_view,
                tag_view.0.clone(),
                resource_view.raw.0.as_slice(),
            ) {
                Err(e) => {
                    eprintln!(
                        "Warn: decode resource {:?} failed, error:{:?}, hex:{}",
                        tag_view.0, e, resource_view.raw
                    );
                }
                Ok(decoded) => resource_view.json = Some(decoded.into()),
            }
        });
    list
}

fn resolve_code(
    state_view: &dyn StateView,
    address: AccountAddress,
    mut list: ListCodeView,
) -> ListCodeView {
    let resolver = ABIResolver::new(state_view);
    list.codes.iter_mut().for_each(|(id, code_view)| {
        let module_id = ModuleId::new(address, id.clone());
        match resolver.resolve_module(&module_id) {
            Err(e) => {
                eprintln!(
                    "Warn: resolve module {:?} failed, error:{:?}, hex:{}",
                    module_id, e, code_view.code
                );
            }
            Ok(abi) => {
                code_view.abi = Some(abi);
            }
        }
    });
    list
}
