// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use starcoin_consensus::{Consensus, G_CRYPTONIGHT};
use std::path::PathBuf;

use crate::cmd_batch_execution::{BatchCmdExec, CmdBatchExecution};
use starcoin_types::block::Block;

#[derive(Debug, Parser)]
#[clap(name = "verify-head", about = "verify head")]
pub struct VerifyHeaderOptions {
    #[clap(long, short = 'i', parse(from_os_str))]
    /// input file, like accounts.csv
    pub input_path: PathBuf,
}

#[derive(Debug)]
pub struct VerifyHeaderError {
    pub block_number: u64,
}

pub struct VerifyHeaderCmdType;

pub fn verify_header_via_export_file(path: PathBuf) -> anyhow::Result<()> {
    let batch_cmd = CmdBatchExecution::new(String::from("verify_block_header"), path, true, 10);
    batch_cmd.progress::<VerifyHeaderCmdType, Block, VerifyHeaderError>()
}

impl BatchCmdExec<VerifyHeaderCmdType, Block, VerifyHeaderError> for Block {
    fn execute(&self) -> (usize, Vec<VerifyHeaderError>) {
        // let header = BlockHeader::new(
        //     HashValue::from_hex_literal(
        //         "0xae1c7990f16e056bbaa7eb82ad0aec905a4ea0c559ca623f13c2a91403f81ecc",
        //     ).unwrap(),
        //     1616847038282,
        //     3,
        //     AccountAddress::from_hex_literal("0x94e957321e7bb2d3eb08ff6be81a6fcd").unwrap(),
        //     HashValue::from_hex_literal(
        //         "0x3da1d80128ea59c683cd1ca88f77b239fb46afa28e9f4b25753b147ca0cefaba",
        //     )
        //         .unwrap(),
        //     HashValue::from_hex_literal(
        //         "0x3df88e7a7b0ae0064fa284f71a3777c76aa83b30f16e8875a5b3ba1d94ca83b1",
        //     )
        //         .unwrap(),
        //     HashValue::from_hex_literal(
        //         "0x610596802d69223d593b5f708e5803c53f1b5958a25097ae7f8fe8cd52ce6e51",
        //     )
        //         .unwrap(),
        //     0,
        //     478.into(),
        //     HashValue::from_hex_literal(
        //         "0xc01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97",
        //     )
        //         .unwrap(),
        //     ChainId::new(251),
        //     2894404328,
        //     BlockHeaderExtra::new([0u8; 4]),
        // );
        G_CRYPTONIGHT
            .verify_header_difficulty(self.header.difficulty(), &self.header)
            .unwrap();
        (1, vec![])
    }
}
