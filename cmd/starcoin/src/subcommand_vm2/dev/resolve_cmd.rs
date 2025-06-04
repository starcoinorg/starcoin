// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use serde::{Serialize, Serializer};
use starcoin_vm2_abi_types::{FunctionABI, ModuleABI, StructInstantiation};
use starcoin_vm2_types::view::{FunctionIdView, ModuleIdView, StructTagView};

/// Resolve Function/Struct/Module get ABI.
#[derive(Debug, Parser)]
#[clap(name = "resolve")]
pub enum ResolveOpt {
    /// dev resolve function 0x1::TransferScripts::peer_to_peer_v2
    Function {
        #[clap()]
        ///function_id like: 0x1::TransferScripts::peer_to_peer_v2
        function_id: FunctionIdView,
    },
    /// dev resolve struct 0x1::Account::Account
    Struct {
        #[clap()]
        ///struct_tag like: 0x1::Account::Account
        struct_tag: StructTagView,
    },
    /// dev resolve module 0x1::Account
    Module {
        #[clap()]
        ///module_id like: 0x1::Account
        module_id: ModuleIdView,
    },
}

pub struct ResolveCommand;

pub enum ResolveResult {
    Function(FunctionABI),
    Struct(StructInstantiation),
    Module(ModuleABI),
}

impl Serialize for ResolveResult {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Function(c) => c.serialize(serializer),
            Self::Struct(r) => r.serialize(serializer),
            Self::Module(r) => r.serialize(serializer),
        }
    }
}

impl CommandAction for ResolveCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ResolveOpt;
    type ReturnItem = ResolveResult;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let rpc_client = ctx.state().vm2()?.client();
        let result = match opt {
            ResolveOpt::Function { function_id } => {
                ResolveResult::Function(rpc_client.contract_resolve_function2(function_id.clone())?)
            }
            ResolveOpt::Struct { struct_tag } => {
                ResolveResult::Struct(rpc_client.contract_resolve_struct2(struct_tag.clone())?)
            }
            ResolveOpt::Module { module_id } => ResolveResult::Module(
                ctx.state()
                    .client()
                    .contract_resolve_module2(module_id.clone())?,
            ),
        };
        Ok(result)
    }
}
