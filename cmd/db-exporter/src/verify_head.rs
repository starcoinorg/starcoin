// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::bail;
use atomic_counter::AtomicCounter;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::task;

use starcoin_chain::verifier::{
    BasicVerifier, ConsensusVerifier, FullVerifier, NoneVerifier, Verifier,
};

use crate::cmd_batch_execution::{BatchCmdExec, BatchProcessError};
use starcoin_types::block::{Block, BlockHeader};

#[derive(Debug, Parser)]
#[clap(name = "verify_head", about = "verify head")]
pub struct VerifyHeadOptions {
    #[clap(long, short = 'i', parse(from_os_str))]
    /// input file, like accounts.csv
    pub input_path: PathBuf,
}
//
// #[derive(Debug)]
// pub struct VerifyHeadError {
//     pub block_number: u64,
//     pub transaction_hash: HashValue,
//     pub error: VMError,
// }
//
// pub struct VerifyHeaderType;
//
// impl BatchCmdExec<BlockHeader, VerifyHeaderType> for BlockHeader {
//     fn execute(&self) -> (usize, Vec<VerifyHeadError>) {
//         (0, vec![])
//     }
// }
