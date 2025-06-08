use clap::Parser;
use move_binary_format::errors::Location;
use starcoin_crypto::HashValue;
use starcoin_types::{block::Block, transaction::TransactionPayload};
use starcoin_vm_types::{errors::VMError, file_format::CompiledModule};
use std::{fmt::Debug, path::PathBuf};
//use starcoin_accumulator::node::AccumulatorStoreType::Block;
use crate::cmd_batch_execution::{BatchCmdExec, CmdBatchExecution};

#[derive(Debug, Parser)]
#[clap(
    name = "verify-modules",
    about = "fast verify all modules, do not execute the transactions"
)]
pub struct VerifyModuleOptions {
    #[clap(long, short = 'i', parse(from_os_str))]
    /// input file, like accounts.csv
    pub input_path: PathBuf,
}

#[derive(Debug)]
pub struct VerifyModuleError {
    pub block_number: u64,
    pub transaction_hash: HashValue,
    pub error: VMError,
}

pub struct VerifyModulesType;

impl BatchCmdExec<VerifyModulesType, Self, VerifyModuleError> for Block {
    fn execute(&self) -> (usize, Vec<VerifyModuleError>) {
        let mut errors = vec![];
        let mut success_modules = 0;
        let block = self;

        for txn in block.transactions() {
            match txn.payload() {
                TransactionPayload::Package(package) => {
                    for module in package.modules() {
                        match CompiledModule::deserialize(module.code()) {
                            Ok(compiled_module) => {
                                match move_bytecode_verifier::verify_module(&compiled_module) {
                                    Err(e) => {
                                        println!(
                                            "verify module block height {}",
                                            block.header().number()
                                        );
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
}

pub fn verify_modules_via_export_file(input_path: PathBuf) -> anyhow::Result<()> {
    let batch_cmd = CmdBatchExecution::new(String::from("verify_module"), input_path, 10);
    batch_cmd.progress::<VerifyModulesType, Block, VerifyModuleError>()
}
