// SPDX-License-Identifier: Apache-2.0
// Copyright (c) The Starcoin Core Contributors

use crate::{CliState, StarcoinOpt};
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use structopt::StructOpt;

/// Some commands for node manager.
#[derive(Debug, StructOpt)]
#[structopt(name = "manager")]
pub enum NodeManagerOpt {
    /// Delete the block and block info of `block-hash`
    /// Note: this command may broken the block database in node.
    #[structopt(name = "delete-block")]
    DeleteBlock {
        #[structopt(name = "block-hash")]
        block_hash: HashValue,
    },
    /// Re execute block of `block-hash` and save result to database, for fix database broken.
    #[structopt(name = "re-execute-block")]
    ReExecuteBlock {
        #[structopt(name = "block-hash")]
        block_hash: HashValue,
    },
    /// Reset the node chain to block of `block-hash`, and clean all blocks after the block.
    /// Note: this command may broken the block database in node.
    #[structopt(name = "reset")]
    Reset {
        #[structopt(name = "block-hash")]
        block_hash: HashValue,
    },
    /// Delete the failed block record from database.
    DeleteFailedBlock {
        #[structopt(name = "block-hash")]
        block_hash: HashValue,
    },
}

pub struct NodeManagerCommand;

impl CommandAction for NodeManagerCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = NodeManagerOpt;
    type ReturnItem = ();

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        match opt {
            NodeManagerOpt::DeleteBlock { block_hash } => {
                client.node_delete_block(*block_hash)?;
            }
            NodeManagerOpt::ReExecuteBlock { block_hash } => {
                client.node_re_execute_block(*block_hash)?;
            }
            NodeManagerOpt::Reset { block_hash } => {
                client.node_reset(*block_hash)?;
            }
            NodeManagerOpt::DeleteFailedBlock { block_hash } => {
                client.node_delete_failed_block(*block_hash)?;
            }
        }

        Ok(())
    }
}
