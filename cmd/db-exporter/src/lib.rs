// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::bail;
use atomic_counter::AtomicCounter;
use indicatif::{ProgressBar, ProgressStyle};
use move_binary_format::errors::Location;
use starcoin_crypto::HashValue;
use starcoin_types::block::Block;
use starcoin_types::transaction::TransactionPayload;
use starcoin_vm_types::errors::VMError;
use starcoin_vm_types::file_format::CompiledModule;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::task;

pub fn verify_modules_via_export_file(input_path: PathBuf) -> anyhow::Result<()> {
    let start_time = SystemTime::now();
    let file_name = input_path.display().to_string();
    let reader = BufReader::new(File::open(input_path)?);
    let mut blocks = vec![];
    for record in reader.lines() {
        let record = record?;
        let block: Block = serde_json::from_str(record.as_str())?;

        blocks.push(block);
    }
    if blocks.is_empty() {
        println!("file {} has apply", file_name);
        return Ok(());
    }

    if let Some(last_block) = blocks.last() {
        let start = blocks.get(0).unwrap().header().number();
        let end = last_block.header().number();
        println!("verify [{},{}] block number", start, end);
    }
    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("load blocks from file use time: {:?}", use_time.as_millis());
    let start_time = SystemTime::now();
    let bar = ProgressBar::new(blocks.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
    );

    let success_counter = Arc::new(atomic_counter::RelaxedCounter::new(0));
    let error_counter = Arc::new(atomic_counter::RelaxedCounter::new(0));
    for block in blocks {
        let total_modules = success_counter.get() + error_counter.get();
        let block_number = block.header().number();
        let success_counter = success_counter.clone();
        let error_counter = error_counter.clone();

        task::spawn(async move {
            let (success_count, errors) = verify_block_modules(block);
            if !errors.is_empty() {
                println!(
                    "verify block modules {} error modules: {:?}",
                    block_number, errors
                );
            }
            success_counter.add(success_count);
            error_counter.add(errors.len());
        });
        bar.set_message(format!(
            "verify block {} , total_modules: {}",
            block_number, total_modules
        ));
        bar.inc(1);
    }
    bar.finish();
    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("verify block modules use time: {:?}, success modules: {}, error modules: {}, total modules: {}", use_time.as_secs(), success_counter.get(), error_counter.get(), success_counter.get() + error_counter.get());
    if error_counter.get() > 0 {
        bail!("verify block modules error");
    }
    Ok(())
}

#[derive(Debug)]
pub struct VerifyModuleError {
    pub block_number: u64,
    pub transaction_hash: HashValue,
    pub error: VMError,
}

fn verify_block_modules(block: Block) -> (usize, Vec<VerifyModuleError>) {
    let mut errors = vec![];
    let mut success_modules = 0;

    for txn in block.transactions() {
        match txn.payload() {
            TransactionPayload::Package(package) => {
                for module in package.modules() {
                    match CompiledModule::deserialize(module.code()) {
                        Ok(compiled_module) => {
                            match move_bytecode_verifier::verify_module(&compiled_module) {
                                Err(e) => {
                                    errors.push(VerifyModuleError {
                                        block_number: block.header().number(),
                                        transaction_hash: txn.id(),
                                        error: e,
                                    });
                                }
                                Ok(_) => {
                                    success_modules += 1;
                                }
                            }
                        }
                        Err(e) => {
                            errors.push(VerifyModuleError {
                                block_number: block.header().number(),
                                transaction_hash: txn.id(),
                                error: e.finish(Location::Undefined),
                            });
                        }
                    }
                }
            }
            TransactionPayload::Script(_) => {
                //TODO
            }
            TransactionPayload::ScriptFunction(_) => {
                //continue
            }
        }
    }
    (success_modules, errors)
}
