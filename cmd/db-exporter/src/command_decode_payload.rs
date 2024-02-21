// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::command_progress::{
    ParallelCommand, ParallelCommandFilter, ParallelCommandObserver, ParallelCommandProgress,
    ParallelCommandReadBlockFromDB,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use csv::{Writer, WriterBuilder};
use serde::Serialize;
use starcoin_abi_decoder;
use starcoin_abi_decoder::DecodedTransactionPayload;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_crypto::{hash::CryptoHash, HashValue};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::Storage;
use starcoin_types::{block::Block, block::BlockNumber, transaction::TransactionPayload};
use starcoin_vm_types::errors::VMError;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};
use std::{fmt::Debug, path::PathBuf};

const DECODE_PAYLOAD_COMMAND_NAME: &str = "decode_payload_command";

#[derive(Debug, Parser)]
#[clap(
    name = "decode-payload",
    about = "Decode payload for given parameter and function name"
)]
pub struct DecodePayloadCommandOptions {
    #[clap(long, short = 'n')]
    /// Chain Network, like main, proxima
    pub net: BuiltinNetworkID,

    #[clap(long, short = 'o', parse(from_os_str))]
    /// output dir, like ~/, output filename like ~/block_start_end.csv
    pub output: PathBuf,

    #[clap(long, short = 'i', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub db_path: PathBuf,

    #[clap(long, short = 's')]
    pub start: Option<BlockNumber>,

    #[clap(long, short = 'e')]
    pub end: Option<BlockNumber>,

    #[clap(long, short = 'c')]
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

// impl DecodePayloadCommandError {
//     fn new_from_vm_error(error: VMError, block_number: u64, txn_hash: &HashValue) -> Self {
//         DecodePayloadCommandError {
//             block_number,
//             txn_hash: txn_hash.clone(),
//             error,
//         }
//     }
//
//     fn new_from_partial_vm_error(
//         error: PartialVMError,
//         block_number: u64,
//         txn_hash: &HashValue,
//     ) -> Self {
//         DecodePayloadCommandError {
//             block_number,
//             txn_hash: txn_hash.clone(),
//             error: error.finish(Location::Undefined),
//         }
//     }
// }

#[derive(Serialize)]
pub struct CSVHeaders {
    block_num: String,
    txn_hash: String,
    signer: String,
    txn_type: String,
    func_name: String,
    ty_args: String,
    args: String,
    timestamp: u64,
    date_time: String,
}

pub struct CommandDecodePayload {
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

fn timestamp_to_datetime(timestamp: u64) -> String {
    // Creates a new SystemTime from the specified number of whole seconds
    let d = UNIX_EPOCH + Duration::from_secs(timestamp);
    // Create DateTime from SystemTime
    let datetime = DateTime::<Utc>::from(d);
    // Formats the combined date and time with the specified format string.
    datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string()
}

impl ParallelCommand<CommandDecodePayload, DecodePayloadCommandError> for Block {
    fn execute(&self, command: &CommandDecodePayload) -> (usize, Vec<DecodePayloadCommandError>) {
        // let errors = vec![];
        // let mut success_module_size = 0;

        let timestamp = self.header.timestamp() / 1000;
        let formatted_date = timestamp_to_datetime(timestamp);

        let root = self.header.state_root();
        let statedb = ChainStateDB::new(command.storage.clone(), Some(root));
        let block_num = self.header.number().to_string();

        for txn in self.transactions() {
            let signer = txn.sender().to_string();
            let decoded_txn_payload =
                starcoin_abi_decoder::decode_txn_payload(&statedb, txn.payload())
                    .expect("Decode transaction payload failed!");

            let mut writer = command.writer_mutex.lock().unwrap();
            match decoded_txn_payload {
                DecodedTransactionPayload::ScriptFunction(payload) => writer
                    .serialize(CSVHeaders {
                        block_num: block_num.clone(),
                        txn_hash: txn.hash().to_string(),
                        txn_type: String::from("ScriptFunction"),
                        signer,
                        func_name: format!("{}::{}", payload.module, payload.function),
                        ty_args: payload
                            .ty_args
                            .iter()
                            .map(|a| a.to_string())
                            .collect::<Vec<_>>()
                            .join("|"),
                        args: payload
                            .args
                            .iter()
                            .map(|a| a.0.to_string())
                            .collect::<Vec<_>>()
                            .join("|"),
                        timestamp,
                        date_time: formatted_date.clone(),
                    })
                    .expect("Write into CSV failed!"),
                DecodedTransactionPayload::Script(_) => writer
                    .serialize(CSVHeaders {
                        block_num: block_num.clone(),
                        txn_hash: txn.hash().to_string(),
                        txn_type: String::from("Script"),
                        signer,
                        func_name: "".to_string(),
                        ty_args: "".to_string(),
                        args: "".to_string(),
                        timestamp,
                        date_time: formatted_date.clone(),
                    })
                    .expect("Write into CSV failed!"),
                DecodedTransactionPayload::Package(_) => writer
                    .serialize(CSVHeaders {
                        block_num: block_num.clone(),
                        txn_hash: txn.hash().to_string(),
                        txn_type: String::from("Package"),
                        signer,
                        func_name: "".to_string(),
                        ty_args: "".to_string(),
                        args: "".to_string(),
                        timestamp,
                        date_time: formatted_date.clone(),
                    })
                    .expect("Write into CSV failed!"),
            }
        }
        //(success_module_size, errors)
        (0, vec![])
    }

    ///
    /// Check whether the conditions are met from the list of all transactions in a block,
    /// and return false if any condition is met.
    ///
    fn matched(&self, filters: &Option<ParallelCommandFilter>) -> bool {
        if self.transactions().is_empty() {
            return true;
        };

        match filters {
            Some(filter) => {
                self.transactions().iter().any(|txn| {
                    match txn.payload() {
                        TransactionPayload::ScriptFunction(payload) => {
                            filter.match_signer(&txn.sender().to_string())
                                && filter.match_func_name(payload.function().as_str())
                                && filter.match_ty_args(payload.ty_args())
                                && filter.match_args(payload.args())
                        },
                        TransactionPayload::Script(_) | TransactionPayload::Package(_) => {
                            filter.match_signer(&txn.sender().to_string())
                        },
                    }
                })
            },
            None => true,
        }
    }
}

pub fn do_decode_payload_command(option: &DecodePayloadCommandOptions) -> Result<()> {
    do_decode_payload(
        option.net,
        option.db_path.clone(),
        option.output.clone(),
        option.start,
        option.end,
        ParallelCommandFilter::new(
            &option.signer,
            &option.func_name,
            &option.ty_args,
            &option.args,
        ),
    )
}

pub fn do_decode_payload(
    net: BuiltinNetworkID,
    input_path: PathBuf,
    out_path: PathBuf,
    start_height: Option<u64>,
    end_height: Option<u64>,
    filter: Option<ParallelCommandFilter>,
) -> Result<()> {
    let file = WriterBuilder::new().from_path(out_path)?;
    let writer_mutex = Mutex::new(file);

    let (dbreader, storage) = ParallelCommandReadBlockFromDB::new(
        input_path,
        ChainNetwork::from(net),
        start_height.unwrap_or(0),
        end_height.unwrap_or(0),
        true,
    )?;
    let command = Arc::new(CommandDecodePayload {
        writer_mutex,
        storage,
    });

    ParallelCommandProgress::new(
        String::from(DECODE_PAYLOAD_COMMAND_NAME),
        num_cpus::get(),
        Arc::new(dbreader),
        filter,
        Some(command.clone() as Arc<dyn ParallelCommandObserver>),
    )
    .progress::<CommandDecodePayload, DecodePayloadCommandError>(&command)
}

#[test]
pub fn test_decode_payload() -> Result<()> {
    let input = PathBuf::from("/Users/bobong/.starcoin/main");
    let output = PathBuf::from("/Users/bobong/Downloads/STC-DB-mainnet/output.csv");

    // do_decode_payload(Main, input.clone(), output.clone(), None, None, None)?;

    // do_decode_payload(
    //     Main,
    //     input.clone(),
    //     output.clone(),
    //     Some(0),
    //     Some(100),
    //     None,
    // )?;
    //
    do_decode_payload(
        BuiltinNetworkID::Main,
        input,
        output,
        None,
        None,
        ParallelCommandFilter::new(
            &Some("0x45b467f509bb82f8bcbd7b01170a22d0".to_string()),
            &None,
            &None,
            &None,
        ),
    )?;

    Ok(())
}
