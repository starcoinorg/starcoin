// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::command_progress::{
    ParallelCommand, ParallelCommandFilter, ParallelCommandObserver, ParallelCommandProgress,
};
use anyhow::Result;
use chrono::{TimeZone, Utc};
use clap::Parser;
use csv::{Writer, WriterBuilder};
use move_binary_format::errors::{Location, PartialVMError};
use serde::Serialize;
use starcoin_crypto::hash::CryptoHash;
use starcoin_crypto::HashValue;
use starcoin_types::{block::Block, transaction::TransactionPayload};
use starcoin_vm_types::errors::VMError;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::{fmt::Debug, path::PathBuf};
use starcoin_abi_decoder;
use starcoin_abi_decoder::{DecodedTransactionPayload};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{Storage};
use crate::init_db_obj;

const DECODE_PAYLOAD_COMMAND_NAME: &str = "decode_payload_command";

#[derive(Debug, Parser)]
#[clap(
name = "decode-payload",
about = "Decode payload for given parameter and function name"
)]
pub struct DecodePayloadCommandOptions {
    #[clap(long, short = 'i', parse(from_os_str))]
    /// input file, like accounts.csv
    pub input_path: PathBuf,

    #[clap(long, short = 'o', parse(from_os_str))]
    /// output file, like accounts.csv
    pub output_path: PathBuf,

    #[clap(long, short = 's')]
    /// Signer filter
    pub signer: Option<String>,

    #[clap(long, short = 'f')]
    /// function name for filter
    pub func_name: Option<String>,

    #[clap(long = "arg", multiple_values = true, number_of_values = 1)]
    /// List of arguments for filter
    pub args: Option<Vec<String>>,

    #[clap(long, short = 't', multiple_values = true, number_of_values = 1)]
    /// List of template arguments for filter
    pub ty_args: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct DecodePayloadCommandError {
    pub block_number: u64,
    pub txn_hash: HashValue,
    pub error: VMError,
}

impl DecodePayloadCommandError {
    fn new_from_vm_error(error: VMError, block_number: u64, txn_hash: &HashValue) -> Self {
        DecodePayloadCommandError {
            block_number,
            txn_hash: txn_hash.clone(),
            error,
        }
    }

    fn new_from_partial_vm_error(
        error: PartialVMError,
        block_number: u64,
        txn_hash: &HashValue,
    ) -> Self {
        DecodePayloadCommandError {
            block_number,
            txn_hash: txn_hash.clone(),
            error: error.finish(Location::Undefined),
        }
    }
}

#[derive(Serialize)]
pub struct CSVHeaders {
    txn_hash: String,
    signer: String,
    func_name: String,
    ty_args: String,
    args: String,
    timestamp: String,
}

pub struct CommandDecodePayload {
    out_path: PathBuf,
    writer_mutex: Mutex<Writer<File>>,
    storage: Arc<Storage>,
}

impl ParallelCommandObserver for CommandDecodePayload {
    fn before_progress(&self) -> Result<()> {
        Ok(())
    }

    fn after_progress(&self) -> Result<()> {
        let mut writer = self.writer_mutex.lock().unwrap();
        writer.flush()?;
        Ok(())
    }
}

impl ParallelCommand<CommandDecodePayload, Block, DecodePayloadCommandError> for Block {
    fn execute(&self, command: &CommandDecodePayload) -> (usize, Vec<DecodePayloadCommandError>) {
        let mut errors = vec![];
        let mut success_module_size = 0;

        let datetime = Utc.timestamp_opt(self.header.timestamp() as i64, 0);
        let formatted_date = datetime.unwrap().format("%Y-%m-%d %H:%M:%s").to_string();

        let root = self.header.state_root();
        let statedb = ChainStateDB::new(command.storage, Some(root));

        for txn in self.transactions() {
            let signer = txn.sender().to_string();
            let decoded_txn_payload = starcoin_abi_decoder::decode_txn_payload(
                &statedb, txn.payload(),
            ).expect("Decode transaction payload failed!");

            match decoded_txn_payload {
                DecodedTransactionPayload::ScriptFunction(payload) => {
                    let mut writer = command.writer_mutex.lock().unwrap();
                    writer.serialize(CSVHeaders {
                        txn_hash: txn.hash().to_string(),
                        signer,
                        func_name: format!("{}::{}", payload.module, payload.function),
                        //ty_args: payload.ty_args.iter().map(|a| a.to_string() + ",").collect(),
                        //args: payload.args.iter().map(|dv| format!("{},", dv.0.as_str()).collect(),
                        ty_args: payload.ty_args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(","),
                        args: payload.args.iter().map(|a| a.0.to_string()).collect::<Vec<_>>().join(","),
                        timestamp: formatted_date.clone(),
                    }).expect("Write into CSV failed!")
                }
                DecodedTransactionPayload::Script(_) | DecodedTransactionPayload::Package(_) => (),
            }
        }
        (success_module_size, errors)
    }

    fn before_command(&self, _cmd: &CommandDecodePayload) -> Result<()> {
        Ok(())
    }

    fn after_command(&self, _cmd: &CommandDecodePayload) -> Result<()> {
        Ok(())
    }

    ///
    /// Check whether the conditions are met from the list of all transactions in a block,
    /// and return false if any condition is met.
    ///
    fn matched(&self, filters: Option<ParallelCommandFilter>) -> bool {
        filters.as_ref().map_or_else(|| true, |f| {
            self.transactions().iter().any(|txn| match txn.payload() {
                TransactionPayload::ScriptFunction(payload) => {
                    f.match_signer(&txn.sender().to_string())
                        || f.match_func_name(payload.function().as_str())
                        || f.match_ty_args(&payload.ty_args().to_vec())
                        || f.match_args(&payload.args().to_vec())
                }
                _ => true,
            })
        })
    }
}


pub fn decode_payload(
    input_path: PathBuf,
    out_path: PathBuf,
    db_path: PathBuf,
    filter: Option<ParallelCommandFilter>,
) -> Result<()> {
    let file = WriterBuilder::new().from_path(out_path.as_ref())?;
    let writer_mutex = Mutex::new(file);

    let command = Arc::new(CommandDecodePayload {
        out_path,
        writer_mutex,
        storage: init_db_obj(db_path)?,
    });

    ParallelCommandProgress::new(
        String::from(DECODE_PAYLOAD_COMMAND_NAME),
        input_path,
        num_cpus::get(),
        filter,
        Some(command.clone() as Arc<dyn ParallelCommandObserver>),
    ).progress::<CommandDecodePayload, Block, DecodePayloadCommandError>(&command)
}

#[test]
pub fn test_decode_payload() {
    let mut workspace = PathBuf::from("/Users/bobong/Downloads/STC-DB-mainnet");
    let mut input = workspace.clone();
    input.push("grep-'LocalPool'.json");

    let mut output = workspace.clone();
    output.push("output.csv");

    decode_payload(input, output, PathBuf::from(""), None)?;
}
