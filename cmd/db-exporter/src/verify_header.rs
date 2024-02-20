// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use starcoin_consensus::{Consensus, G_CRYPTONIGHT};
use std::path::PathBuf;

use crate::command_progress::{ParallelCommand, ParallelCommandFilter, ParallelCommandProgress};
use starcoin_types::block::Block;

#[derive(Debug, Parser)]
#[clap(name = "verify-head", about = "verify head")]
pub struct VerifyHeaderOptions {
    #[clap(long, short = 'i', parse(from_os_str))]
    /// input file, like accounts.csv
    pub input_path: PathBuf,

    #[clap(short, long, default_value = "16")]
    /// batch size to do transfer
    pub batch_size: usize,
}

#[derive(Debug)]
pub struct VerifyHeaderError {
    pub block_number: u64,
}

pub struct VerifyHeaderCmdType;

pub fn verify_header_via_export_file(path: PathBuf, batch_size: usize) -> anyhow::Result<()> {
    let batch_cmd = ParallelCommandProgress::new(
        String::from("verify_block_header"),
        path,
        batch_size,
        None,
        None,
    );
    batch_cmd.progress::<VerifyHeaderCmdType, Block, VerifyHeaderError>(&VerifyHeaderCmdType {})
}

impl ParallelCommand<VerifyHeaderCmdType, Block, VerifyHeaderError> for Block {
    fn execute(&self, _cmd: &VerifyHeaderCmdType) -> (usize, Vec<VerifyHeaderError>) {
        let ret = G_CRYPTONIGHT.verify_header_difficulty(self.header.difficulty(), &self.header);
        match ret {
            Ok(_) => (1, vec![]),
            Err(_) => {
                println!("Failed for block, block num: {} ", self.header.number());
                (
                    0,
                    vec![VerifyHeaderError {
                        block_number: self.header.number(),
                    }],
                )
            }
        }
    }

    fn matched(&self, _filter: &Option<ParallelCommandFilter>) -> bool {
        true
    }
}
