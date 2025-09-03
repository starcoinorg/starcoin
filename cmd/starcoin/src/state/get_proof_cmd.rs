// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use serde::Serialize;
use starcoin_crypto::HashValue;
use starcoin_vm2_state_api::StateWithProof;
use starcoin_vm2_types::access_path::AccessPath;
use starcoin_vm2_types::view::{StateWithProofView, StrView};
use starcoin_vm2_vm_types::access_path::DataPath;
use starcoin_vm2_vm_types::state_store::state_key::StateKey;

/// Get state and proof with access_path, etc: 0x1/0/Account,  0x1/1/0x1::Account::Account
#[derive(Debug, Parser)]
#[clap(name = "get-proof", alias = "get_proof")]
pub struct GetProofOpt {
    #[clap(name = "access_path")]
    /// access_path of code or resource, etc: 0x1/0/Account,  0x1/1/0x1::Account::Account
    access_path: AccessPath,
    #[clap(name = "state-root", long)]
    /// state_root of the proof
    state_root: Option<HashValue>,
    #[clap(name = "raw", long)]
    /// Return raw hex string of state proof
    raw: bool,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ViewOrRaw {
    View(StateWithProofView),
    Raw(StrView<Vec<u8>>),
}

impl Serialize for ViewOrRaw {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::View(v) => v.serialize(serializer),
            Self::Raw(v) => v.serialize(serializer),
        }
    }
}

pub struct GetProofCommand;

impl CommandAction for GetProofCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetProofOpt;
    type ReturnItem = ViewOrRaw;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let state_root = match opt.state_root {
            Some(v) => v,
            None => client.state_get_state_root2()?,
        };
        let access_path = opt.access_path.clone();
        let state_key = match access_path.clone().path {
            DataPath::Code(module_name) => StateKey::module(&access_path.address, &module_name),
            DataPath::Resource(struct_tag) => {
                StateKey::resource(&access_path.address, &struct_tag)?
            }
            DataPath::ResourceGroup(struct_tag) => {
                StateKey::resource_group(&access_path.address, &struct_tag)
            }
        };
        let (proof, result) = if opt.raw {
            let proof = client.state_get_with_proof_by_root_raw2(state_key, state_root)?;
            (
                bcs_ext::from_bytes::<StateWithProof>(proof.0.as_slice())?,
                ViewOrRaw::Raw(proof),
            )
        } else {
            let proof = client.state_get_with_proof_by_root2(state_key, state_root)?;
            (proof.clone().into(), ViewOrRaw::View(proof))
        };
        proof.verify(state_root, access_path)?;
        Ok(result)
    }
}
