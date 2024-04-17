// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use bcs_ext::{BCSCodec, Sample};
use clap::{IntoApp, Parser};
use csv::Writer;
use db_exporter::force_deploy_output::{force_deploy_output, ForceDeployOutput};
use db_exporter::{
    verify_header::{verify_header_via_export_file, VerifyHeaderOptions},
    verify_module::{verify_modules_via_export_file, VerifyModuleOptions},
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::{ser::SerializeMap, Serialize, Serializer};
use starcoin_account_api::AccountInfo;
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
use starcoin_chain::{
    verifier::{BasicVerifier, ConsensusVerifier, FullVerifier, NoneVerifier, Verifier},
    BlockChain, ChainReader, ChainWriter,
};
use starcoin_config::{BuiltinNetworkID, ChainNetwork, RocksdbConfig};
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_genesis::Genesis;
use starcoin_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue, MoveValueAnnotator};
use starcoin_rpc_api::types::StrView;
use starcoin_state_tree::StateTree;
use starcoin_statedb::{ChainStateDB, ChainStateReader, ChainStateWriter};
use starcoin_storage::{
    block::FailedBlock,
    block_info::BlockInfoStore,
    cache_storage::CacheStorage,
    db_storage::DBStorage,
    storage::{ColumnFamilyName, InnerStore, StorageInstance, ValueCodec},
    BlockStore, Storage, StorageVersion, Store, BLOCK_ACCUMULATOR_NODE_PREFIX_NAME,
    BLOCK_HEADER_PREFIX_NAME, BLOCK_INFO_PREFIX_NAME, BLOCK_PREFIX_NAME, FAILED_BLOCK_PREFIX_NAME,
    STATE_NODE_PREFIX_NAME, STATE_NODE_PREFIX_NAME_PREV, TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME,
};
use starcoin_transaction_builder::{
    build_signed_empty_txn, create_signed_txn_with_association_account, DEFAULT_MAX_GAS_AMOUNT,
};
use starcoin_types::{
    account::{peer_to_peer_txn, Account, DEFAULT_EXPIRATION_TIME},
    account_address::AccountAddress,
    account_state::AccountState,
    block::{Block, BlockHeader, BlockInfo, BlockNumber},
    startup_info::{SnapshotRange, StartupInfo},
    state_set::{AccountStateSet, ChainStateSet},
    transaction::Transaction,
};
use starcoin_vm_runtime::starcoin_vm::StarcoinVM;
use starcoin_vm_types::{
    access_path::DataType,
    account_config::stc_type_tag,
    genesis_config::ConsensusStrategy,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag},
    parser::parse_struct_tag,
    transaction::{ScriptFunction, SignedUserTransaction, TransactionPayload},
};
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
    thread::JoinHandle,
    time::SystemTime,
};

const BLOCK_GAP: u64 = 1000;
const BACK_SIZE: u64 = 10000;
const SNAP_GAP: u64 = 128;
const BATCH_SIZE: u64 = 1000;

pub fn export<W: std::io::Write>(
    db: &str,
    mut csv_writer: Writer<W>,
    schema: DbSchema,
) -> anyhow::Result<()> {
    let db_storage = DBStorage::open_with_cfs(
        db,
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    let mut iter = db_storage.iter::<Vec<u8>, Vec<u8>>(schema.to_string().as_str())?;
    iter.seek_to_first();
    let key_codec = schema.get_key_codec();
    let value_codec = schema.get_value_codec();
    let fields = schema.get_fields();
    // write csv header.
    {
        csv_writer.write_field("key")?;
        for field in fields.as_slice() {
            csv_writer.write_field(field)?;
        }
        csv_writer.write_record(None::<&[u8]>)?;
    }

    for item in iter {
        let (k, v) = item?;
        let key = key_codec(k);
        let value = value_codec(v)?;
        let object = value.as_object().expect("should be object.");

        let mut record = vec![key];
        for field in fields.as_slice() {
            let field_value: Option<&serde_json::Value> = object.get(field);
            match field_value {
                Some(value) => {
                    let record_field = match value {
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => "null".to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        value => serde_json::to_string(value)?,
                    };
                    record.push(record_field);
                }
                None => {
                    record.push("null".to_string());
                }
            }
        }

        csv_writer.serialize(record)?;
    }
    // flush csv writer
    csv_writer.flush()?;
    Ok(())
}

#[derive(Debug, Copy, Clone)]
pub enum DbSchema {
    Block,
    BlockHeader,
    FailedBlock,
}

impl DbSchema {
    pub fn get_key_codec(&self) -> Box<dyn Fn(Vec<u8>) -> String> {
        Box::new(|arg| -> String { hex::encode(arg) })
    }

    pub fn get_fields(&self) -> Vec<String> {
        let sample_json = match self {
            DbSchema::Block => {
                serde_json::to_value(Block::sample()).expect("block to json should success")
            }
            DbSchema::BlockHeader => serde_json::to_value(BlockHeader::sample())
                .expect("block header to json should success"),
            DbSchema::FailedBlock => serde_json::to_value(FailedBlock::sample())
                .expect("block header to json should success"),
        };
        sample_json
            .as_object()
            .expect("should be object")
            .keys()
            .cloned()
            .collect()
    }

    pub fn get_value_codec(&self) -> Box<dyn Fn(Vec<u8>) -> Result<serde_json::Value>> {
        Box::new(match self {
            DbSchema::Block => |arg| -> Result<serde_json::Value> {
                Ok(serde_json::to_value(Block::decode_value(arg.as_slice())?)?)
            },
            DbSchema::BlockHeader => |arg| -> Result<serde_json::Value> {
                Ok(serde_json::to_value(BlockHeader::decode_value(
                    arg.as_slice(),
                )?)?)
            },
            DbSchema::FailedBlock => |arg| -> Result<serde_json::Value> {
                Ok(serde_json::to_value(FailedBlock::decode_value(
                    arg.as_slice(),
                )?)?)
            },
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            DbSchema::Block => BLOCK_PREFIX_NAME,
            DbSchema::BlockHeader => BLOCK_HEADER_PREFIX_NAME,
            DbSchema::FailedBlock => FAILED_BLOCK_PREFIX_NAME,
        }
    }
}

impl std::fmt::Display for DbSchema {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for DbSchema {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let schema = match s {
            BLOCK_PREFIX_NAME => DbSchema::Block,
            BLOCK_HEADER_PREFIX_NAME => DbSchema::BlockHeader,
            FAILED_BLOCK_PREFIX_NAME => DbSchema::FailedBlock,
            _ => {
                bail!("Unsupported schema: {}", s)
            }
        };
        Ok(schema)
    }
}

#[derive(Parser)]
struct Opt {
    #[clap(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Parser)]
enum Cmd {
    Exporter(ExporterOptions),
    Checkkey(CheckKeyOptions),
    ExportBlockRange(ExportBlockRangeOptions),
    ApplyBlock(ApplyBlockOptions),
    StartupInfoBack(StartupInfoBackOptions),
    GenBlockTransactions(GenBlockTransactionsOptions),
    ExportSnapshot(ExportSnapshotOptions),
    ApplySnapshot(ApplySnapshotOptions),
    ExportResource(ExportResourceOptions),
    VerifyModules(VerifyModuleOptions),
    VerifyHeader(VerifyHeaderOptions),
    GenTurboSTMTransactions(GenTurboSTMTransactionsOptions),
    ApplyTurboSTMBlock(ApplyTurboSTMBlockOptions),
    VerifyBlock(VerifyBlockOptions),
    BlockOutput(BlockOutputOptions),
    ApplyBlockOutput(ApplyBlockOutputOptions),
    SaveStartupInfo(SaveStartupInfoOptions),
    TokenSupply(TokenSupplyOptions),
    ForceDeploy(ForceDeployOutput),
}

#[derive(Debug, Clone, Parser)]
#[clap(name = "db-exporter", about = "starcoin db exporter")]
pub struct ExporterOptions {
    #[clap(long, short = 'o', parse(from_os_str))]
    /// output file, like accounts.csv, default is stdout.
    pub output: Option<PathBuf>,
    #[clap(long, short = 'i', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/barnard/starcoindb/db/starcoindb
    pub db_path: PathBuf,

    #[clap(long, short = 's')]
    /// the table of database which to export, block,block_header
    pub schema: DbSchema,
}

#[derive(Debug, Clone, Parser)]
#[clap(name = "checkkey", about = "starcoin db check key")]
pub struct CheckKeyOptions {
    #[clap(long, short = 'i', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/barnard/starcoindb/db/starcoindb
    pub db_path: PathBuf,
    #[clap(long, short = 'n',
    possible_values=&["block", "block_header"],)]
    pub cf_name: String,
    #[clap(long, short = 'b')]
    pub block_hash: HashValue,
}

#[derive(Debug, Clone, Parser)]
#[clap(name = "export-block-range", about = "export block range")]
pub struct ExportBlockRangeOptions {
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
    pub start: BlockNumber,
    #[clap(long, short = 'e')]
    pub end: BlockNumber,
}

#[derive(Debug, Parser)]
#[clap(name = "apply-block-range", about = "apply block range")]
pub struct ApplyBlockOptions {
    #[clap(long, short = 'n')]
    /// Chain Network
    pub net: BuiltinNetworkID,
    #[clap(long, short = 'o', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub to_path: PathBuf,
    #[clap(long, short = 'i', parse(from_os_str))]
    /// input file, like accounts.csv
    pub input_path: PathBuf,
    #[clap(possible_values = Verifier::variants(), ignore_case = true)]
    /// Verify type:  Basic, Consensus, Full, None, eg.
    pub verifier: Option<Verifier>,
    #[clap(long, short = 'w')]
    /// Watch metrics logs.
    pub watch: bool,
}

#[derive(Debug, Parser)]
#[clap(name = "startup_info_back", about = "startup info back")]
pub struct StartupInfoBackOptions {
    #[clap(long, short = 'n')]
    /// Chain Network
    pub net: BuiltinNetworkID,
    #[clap(long, short = 'o', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub to_path: PathBuf,
    /// startupinfo BlockNumber back off size
    #[clap(long, short = 'b')]
    pub back_size: Option<u64>,
}

#[derive(Debug, Copy, Clone)]
pub enum Txntype {
    CreateAccount,
    FixAccount,
    EmptyTxn,
}

impl FromStr for Txntype {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let txn_type = match s {
            "CreateAccount" => Txntype::CreateAccount,
            "FixAccount" => Txntype::FixAccount,
            "EmptyTxn" => Txntype::EmptyTxn,
            _ => {
                bail!("Unsupported TxnType: {}", s)
            }
        };
        Ok(txn_type)
    }
}

#[derive(Debug, Parser)]
#[clap(name = "gen_block_transactions", about = "gen block transactions")]
pub struct GenBlockTransactionsOptions {
    #[clap(long, short = 'o', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/halley
    pub to_path: PathBuf,
    #[clap(long, short = 'b')]
    pub block_num: Option<u64>,
    #[clap(long, short = 't')]
    pub trans_num: Option<u64>,
    #[clap(long, short = 'p', possible_values=&["CreateAccount", "FixAccount", "EmptyTxn"],)]
    /// txn type
    pub txn_type: Txntype,
}

#[derive(Debug, Clone, Parser)]
#[clap(name = "export-snapshot", about = "export snapshot")]
pub struct ExportSnapshotOptions {
    #[clap(long, short = 'n')]
    /// Chain Network, like main, proxima
    pub net: BuiltinNetworkID,
    #[clap(long, short = 'o', parse(from_os_str))]
    /// output dir, like ~/, manifest.csv will write in output dir
    pub output: PathBuf,
    #[clap(long, short = 'i', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub db_path: PathBuf,
    #[clap(long, short = 't')]
    /// enable increment export snapshot
    pub increment: Option<bool>,
    #[clap(long, short = 'b')]
    /// special block_num for debug usage
    pub special_block_num: Option<BlockNumber>,
}

#[derive(Debug, Parser)]
#[clap(name = "apply-snapshot", about = "apply snapshot")]
pub struct ApplySnapshotOptions {
    #[clap(long, short = 'n')]
    /// Chain Network
    pub net: BuiltinNetworkID,
    #[clap(long, short = 'o', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub to_path: PathBuf,
    #[clap(long, short = 'i', parse(from_os_str))]
    /// input_path, manifest.csv in this dir
    pub input_path: PathBuf,
}

#[derive(Debug, Clone, Parser)]
#[clap(name = "export-resource", about = "onchain resource exporter")]
pub struct ExportResourceOptions {
    #[clap(long, short = 'o', parse(from_os_str))]
    /// output file, like accounts.csv
    pub output: PathBuf,
    #[clap(long, short = 'i', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/barnard/starcoindb/db/starcoindb
    pub db_path: PathBuf,

    #[clap(long)]
    /// block hash of the snapshot.
    pub block_hash: HashValue,

    #[clap(
        short='r',
        default_value = "0x1::Account::Balance<0x1::STC::STC>",
        parse(try_from_str=parse_struct_tag)
    )]
    /// resource struct tag.
    resource_type: StructTag,

    #[clap(min_values = 1, required = true)]
    /// fields of the struct to output. it use pointer syntax of serde_json.
    /// like: /authentication_key /sequence_number /deposit_events/counter /token/value
    pub fields: Vec<String>,
}

#[derive(Debug, Parser)]
#[clap(
    name = "gen_turbo_stm_transactions",
    about = "gen turbo stm transactions"
)]
pub struct GenTurboSTMTransactionsOptions {
    #[clap(long, short = 'o', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/halley
    pub to_path: PathBuf,
    #[clap(long, short = 'b')]
    pub block_num: Option<u64>,
}

#[derive(Debug, Parser)]
#[clap(name = "apply turbo stm block", about = "apply turbo stm block")]
pub struct ApplyTurboSTMBlockOptions {
    #[clap(long, short = 'o', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/test
    pub to_path: PathBuf,
    #[clap(long, short = 't', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/test_stm
    pub turbo_stm_to_path: PathBuf,
    #[clap(long, short = 'i', parse(from_os_str))]
    /// input file, like accounts.csv
    pub input_path: PathBuf,
}

#[derive(Debug, Parser)]
#[clap(name = "verify-block-range", about = "verify block range")]
pub struct VerifyBlockOptions {
    #[clap(long, short = 'n')]
    /// Chain Network
    pub net: BuiltinNetworkID,
    #[clap(long, short = 'i', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub from_path: PathBuf,
    #[clap(possible_values = Verifier::variants(), ignore_case = true)]
    /// Verify type:  Basic, Consensus, Full, None, eg.
    pub verifier: Option<Verifier>,
    #[clap(long, short = 's')]
    pub start: BlockNumber,
    #[clap(long, short = 'e')]
    pub end: Option<BlockNumber>,
}

#[derive(Debug, Parser)]
#[clap(name = "block-output", about = "block output options")]
pub struct BlockOutputOptions {
    #[clap(long, short = 'n')]
    /// Chain Network
    pub net: BuiltinNetworkID,
    #[clap(long, short = 'i', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub from_path: PathBuf,
    #[clap(long, short = 's')]
    pub num: BlockNumber,
}

#[derive(Debug, Parser)]
#[clap(name = "apply-block-output", about = "apply block output")]
pub struct ApplyBlockOutputOptions {
    #[clap(long, short = 'n')]
    /// Chain Network
    pub net: BuiltinNetworkID,
    #[clap(long, short = 'o', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub to_path: PathBuf,
    #[clap(long, short = 'i', parse(from_os_str))]
    /// input file, like accounts.csv
    pub input_path: PathBuf,
}

#[derive(Debug, Parser)]
#[clap(name = "save_startup_info", about = "save startup info")]
pub struct SaveStartupInfoOptions {
    #[clap(long, short = 'n')]
    /// Chain Network
    pub net: BuiltinNetworkID,
    #[clap(long, short = 'o', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub to_path: PathBuf,
    /// startupinfo BlockNumber back off size
    #[clap(long, short = 'b')]
    pub hash_value: HashValue,
}

#[derive(Debug, Clone, Parser)]
#[clap(name = "token-supply", about = "token supply")]
pub struct TokenSupplyOptions {
    #[clap(long, short = 'n')]
    /// Chain Network, like main, barnard
    pub net: BuiltinNetworkID,
    #[clap(long, short = 'o', parse(from_os_str))]
    /// output file, like balance.csv
    pub output: PathBuf,
    #[clap(long, short = 'i', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub db_path: PathBuf,

    #[clap(long, short = 'b')]
    pub block_number: Option<BlockNumber>,

    #[clap(
        help = "resource struct tag,",
        default_value = "0x1::Account::Balance<0x1::STC::STC>"
    )]
    resource_type: StrView<StructTag>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    let cmd = match opt.cmd {
        Some(cmd) => cmd,
        None => {
            Opt::command().print_help().ok();
            return Ok(());
        }
    };

    match cmd {
        Cmd::Exporter(option) => {
            let output = option.output.as_deref();
            let mut writer_builder = csv::WriterBuilder::new();
            let writer_builder = writer_builder.delimiter(b'\t').double_quote(false);
            let result = match output {
                Some(output) => {
                    let writer = writer_builder.from_path(output)?;
                    export(
                        // option.db_path.display().to_string().as_str(),
                        option.db_path.to_str().unwrap(),
                        writer,
                        option.schema,
                    )
                }
                None => {
                    let writer = writer_builder.from_writer(std::io::stdout());
                    export(
                        option.db_path.display().to_string().as_str(),
                        writer,
                        option.schema,
                    )
                }
            };
            if let Err(err) = result {
                let broken_pipe_err = err.downcast_ref::<csv::Error>().and_then(|err| {
                    if let csv::ErrorKind::Io(io_err) = err.kind() {
                        if io_err.kind() == std::io::ErrorKind::BrokenPipe {
                            Some(io_err)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });
                //ignore BrokenPipe
                return if let Some(_broken_pipe_err) = broken_pipe_err {
                    Ok(())
                } else {
                    Err(err)
                };
            }
            return result;
        }
        Cmd::Checkkey(option) => {
            let db = DBStorage::open_with_cfs(
                option.db_path.display().to_string().as_str(),
                StorageVersion::current_version()
                    .get_column_family_names()
                    .to_vec(),
                true,
                Default::default(),
                None,
            )?;

            let result = db.get(option.cf_name.as_str(), option.block_hash.to_vec())?;
            if result.is_some() {
                println!("{} block_hash {} exist", option.cf_name, option.block_hash);
            } else {
                println!(
                    "{} block_hash {} not exist",
                    option.cf_name, option.block_hash
                );
            }
            return Ok(());
        }
        Cmd::ExportBlockRange(option) => {
            let result = export_block_range(
                option.db_path,
                option.output,
                option.net,
                option.start,
                option.end,
            );
            return result;
        }
        Cmd::ApplyBlock(option) => {
            #[cfg(target_os = "linux")]
            let guard = pprof::ProfilerGuard::new(100).unwrap();
            let verifier = option.verifier.unwrap_or(Verifier::Basic);
            let result = apply_block(option.to_path, option.input_path, option.net, verifier);
            #[cfg(target_os = "linux")]
            if let Ok(report) = guard.report().build() {
                let file = File::create("/tmp/flamegraph.svg").unwrap();
                report.flamegraph(file).unwrap();
            }
            return result;
        }
        Cmd::StartupInfoBack(option) => {
            let result = startup_info_back(option.to_path, option.back_size, option.net);
            return result;
        }
        Cmd::GenBlockTransactions(option) => {
            let result = gen_block_transactions(
                option.to_path,
                option.block_num,
                option.trans_num,
                option.txn_type,
            );
            return result;
        }
        Cmd::ExportSnapshot(option) => {
            let result = export_snapshot(
                option.db_path,
                option.output,
                option.net,
                option.increment,
                option.special_block_num,
            );
            return result;
        }
        Cmd::ApplySnapshot(option) => {
            let result = apply_snapshot(option.to_path, option.input_path, option.net);
            return result;
        }
        Cmd::ExportResource(option) => {
            #[cfg(target_os = "linux")]
            let guard = pprof::ProfilerGuard::new(100).unwrap();
            let output = option.output.as_path();
            let block_hash = option.block_hash;
            let resource = option.resource_type.clone();
            // let result = apply_block(option.to_path, option.input_path, option.net, verifier);
            export_resource(
                option.db_path.display().to_string().as_str(),
                output,
                block_hash,
                resource,
                option.fields.as_slice(),
            )?;
            #[cfg(target_os = "linux")]
            if let Ok(report) = guard.report().build() {
                let file = File::create("/tmp/flamegraph-db-export-resource-freq-100.svg").unwrap();
                report.flamegraph(file).unwrap();
            }
        }
        Cmd::VerifyModules(option) => {
            return verify_modules_via_export_file(option.input_path);
        }

        Cmd::VerifyHeader(option) => {
            return verify_header_via_export_file(option.input_path, option.batch_size);
        }
        Cmd::GenTurboSTMTransactions(option) => {
            let result = gen_turbo_stm_transactions(option.to_path, option.block_num);
            return result;
        }
        Cmd::ApplyTurboSTMBlock(option) => {
            let result =
                apply_turbo_stm_block(option.to_path, option.turbo_stm_to_path, option.input_path);
            return result;
        }
        Cmd::VerifyBlock(option) => {
            let verifier = option.verifier.unwrap_or(Verifier::Basic);
            let result = verify_block(
                option.from_path,
                option.net,
                option.start,
                option.end,
                verifier,
            );
            return result;
        }
        Cmd::BlockOutput(option) => {
            let result = block_output(option.from_path, option.net, option.num);
            return result;
        }
        Cmd::ApplyBlockOutput(option) => {
            let result = apply_block_output(option.to_path, option.input_path, option.net);
            return result;
        }
        Cmd::ForceDeploy(option) => {
            return force_deploy_output(
                option.input_path,
                option.package_path,
                option.net,
                option.block_num,
            )
        }
        Cmd::SaveStartupInfo(option) => {
            let result = save_startup_info(option.to_path, option.net, option.hash_value);
            return result;
        }
        Cmd::TokenSupply(option) => {
            let result = token_supply(
                option.db_path,
                option.output,
                option.net,
                option.block_number,
                option.resource_type.0,
            );
            return result;
        }
    }
    Ok(())
}

pub fn export_block_range(
    from_dir: PathBuf,
    output: PathBuf,
    network: BuiltinNetworkID,
    start: BlockNumber,
    end: BlockNumber,
) -> anyhow::Result<()> {
    let net = ChainNetwork::new_builtin(network);
    let db_storage = DBStorage::open_with_cfs(
        from_dir.join("starcoindb/db/starcoindb"),
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (chain_info, _) =
        Genesis::init_and_check_storage(&net, storage.clone(), from_dir.as_ref())?;
    let chain = BlockChain::new(net.time_service(), chain_info.head().id(), storage, None)
        .expect("create block chain should success.");
    let cur_num = chain.status().head().number();
    let end = if cur_num > end + BLOCK_GAP {
        end
    } else if cur_num > BLOCK_GAP {
        cur_num - BLOCK_GAP
    } else {
        end
    };
    if start > cur_num || start > end {
        return Err(format_err!(
            "cur_num {} start {} end {} illegal",
            cur_num,
            start,
            end
        ));
    }
    let start_time = SystemTime::now();
    let total = end - start + 1;
    let load_bar = ProgressBar::new(total);
    load_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
    );

    let block_list: Result<Vec<Block>> = (start..=end)
        .collect::<Vec<BlockNumber>>()
        .into_iter()
        .map(|num| {
            load_bar.set_message(format!("load block {}", num));
            load_bar.inc(1);
            chain
                .get_block_by_number(num)?
                .ok_or_else(|| format_err!("{} get block error", num))
        })
        .collect();
    load_bar.finish();
    let block_list = block_list?;
    let filename = format!("block_{}_{}.csv", start, end);
    let mut file = File::create(output.join(filename))?;
    let bar = ProgressBar::new(end - start + 1);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
    );
    for block in block_list {
        writeln!(file, "{}", serde_json::to_string(&block)?)?;
        bar.set_message(format!("write block {}", block.header().number()));
        bar.inc(1);
    }
    file.flush()?;
    bar.finish();
    let use_time = SystemTime::now().duration_since(start_time)?;
    println!(
        "export range block [{}..{}] use time: {:?}",
        start,
        end,
        use_time.as_secs()
    );
    Ok(())
}

pub fn apply_block(
    to_dir: PathBuf,
    input_path: PathBuf,
    network: BuiltinNetworkID,
    verifier: Verifier,
) -> anyhow::Result<()> {
    ::starcoin_logger::init();
    let net = ChainNetwork::new_builtin(network);
    let db_storage = DBStorage::new(to_dir.join("starcoindb/db"), RocksdbConfig::default(), None)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    // StarcoinVM::set_concurrency_level_once(num_cpus::get());
    let (chain_info, _) = Genesis::init_and_check_storage(&net, storage.clone(), to_dir.as_ref())?;
    let mut chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage.clone(),
        None,
    )
    .expect("create block chain should success.");
    let start_time = SystemTime::now();
    let cur_num = chain.status().head().number();
    let file_name = input_path.display().to_string();
    let reader = BufReader::new(File::open(input_path)?);
    let mut blocks = vec![];
    for record in reader.lines() {
        let record = record?;
        let block: Block = serde_json::from_str(record.as_str())?;
        if block.header().number() <= cur_num {
            continue;
        }
        blocks.push(block);
    }
    if blocks.is_empty() {
        println!("file {} has apply", file_name);
        return Ok(());
    }

    if let Some(last_block) = blocks.last() {
        let start = blocks.get(0).unwrap().header().number();
        let end = last_block.header().number();
        println!(
            "current number {}, import [{},{}] block number",
            cur_num, start, end
        );
    }
    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("load blocks from file use time: {:?}", use_time.as_millis());
    let start_time = SystemTime::now();
    let bar = ProgressBar::new(blocks.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
    );
    for block in blocks {
        let block_hash = block.header().id();
        let block_number = block.header().number();
        match verifier {
            Verifier::Basic => chain.apply_with_verifier::<BasicVerifier>(block)?,
            Verifier::Consensus => chain.apply_with_verifier::<ConsensusVerifier>(block)?,
            Verifier::Full => chain.apply_with_verifier::<FullVerifier>(block)?,
            Verifier::None => chain.apply_with_verifier::<NoneVerifier>(block)?,
        };
        // apply block then flush startup_info for breakpoint resume
        let startup_info = StartupInfo::new(block_hash);
        storage.save_startup_info(startup_info)?;
        bar.set_message(format!("apply block {}", block_number));
        bar.inc(1);
    }
    bar.finish();
    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("apply block use time: {:?}", use_time.as_secs());
    let chain_info = storage
        .get_chain_info()?
        .ok_or_else(|| format_err!("{}", "get chain_info error"))?;
    println!("chain_info {}", chain_info);
    Ok(())
}

pub fn startup_info_back(
    to_dir: PathBuf,
    back_size: Option<u64>,
    network: BuiltinNetworkID,
) -> anyhow::Result<()> {
    let net = ChainNetwork::new_builtin(network);
    let db_storage = DBStorage::new(to_dir.join("starcoindb/db"), RocksdbConfig::default(), None)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (chain_info, _) = Genesis::init_and_check_storage(&net, storage.clone(), to_dir.as_ref())?;
    let chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage.clone(),
        None,
    )
    .expect("create block chain should success.");

    let cur_num = chain.status().head().number();
    let back_size = back_size.unwrap_or(BACK_SIZE);
    if cur_num <= back_size {
        println!(
            "startup_info block number {} <= back_size {}",
            cur_num, back_size
        );
        return Ok(());
    }
    let block_number = cur_num - back_size;
    let block = chain
        .get_block_by_number(block_number)?
        .ok_or_else(|| format_err!("{} get block error", block_number))?;
    let block_hash = block.header().id();
    let startup_info = StartupInfo::new(block_hash);
    storage.save_startup_info(startup_info)?;
    println!(
        "startup_info block number origin {} now  {}",
        cur_num, block_number
    );
    Ok(())
}

pub fn gen_block_transactions(
    to_dir: PathBuf,
    block_num: Option<u64>,
    trans_num: Option<u64>,
    txn_type: Txntype,
) -> anyhow::Result<()> {
    starcoin_logger::init();
    let net = ChainNetwork::new_builtin(BuiltinNetworkID::Halley);
    let db_storage = DBStorage::new(to_dir.join("starcoindb/db"), RocksdbConfig::default(), None)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (chain_info, _) = Genesis::init_and_check_storage(&net, storage.clone(), to_dir.as_ref())?;
    let mut chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage.clone(),
        None,
    )
    .expect("create block chain should success.");
    let block_num = block_num.unwrap_or(1000);
    let trans_num = trans_num.unwrap_or(200);
    match txn_type {
        Txntype::CreateAccount => execute_transaction_with_miner_create_account(
            storage, &mut chain, &net, block_num, trans_num,
        ),
        Txntype::FixAccount => {
            execute_transaction_with_fixed_account(storage, &mut chain, &net, block_num, trans_num)
        }
        Txntype::EmptyTxn => {
            execute_empty_transaction_with_miner(storage, &mut chain, &net, block_num, trans_num)
        }
    }
}
/// Returns a transaction to create a new account with the given arguments.
pub fn create_account_txn_sent_as_association(
    new_account: &Account,
    seq_num: u64,
    initial_amount: u128,
    expiration_timstamp_secs: u64,
    net: &ChainNetwork,
) -> SignedUserTransaction {
    let args = vec![
        bcs_ext::to_bytes(new_account.address()).unwrap(),
        bcs_ext::to_bytes(&new_account.auth_key().to_vec()).unwrap(),
        bcs_ext::to_bytes(&initial_amount).unwrap(),
    ];

    create_signed_txn_with_association_account(
        TransactionPayload::ScriptFunction(ScriptFunction::new(
            ModuleId::new(
                starcoin_vm_types::account_config::core_code_address(),
                Identifier::new("Account").unwrap(),
            ),
            Identifier::new("create_account_with_initial_amount").unwrap(),
            vec![stc_type_tag()],
            args,
        )),
        seq_num,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        expiration_timstamp_secs,
        net,
    )
}

// This use in test net create account then transfer faster then transfer non exist account
pub fn execute_transaction_with_create_account(
    storage: Arc<Storage>,
    chain: &mut BlockChain,
    net: &ChainNetwork,
    block_num: u64,
    trans_num: u64,
) -> anyhow::Result<()> {
    let mut sequence = 0u64;
    for _i in 0..block_num {
        let mut txns = Vec::with_capacity(20);
        let miner_account = Account::new();
        let miner_info = AccountInfo::from(&miner_account);
        let txn = Transaction::UserTransaction(create_account_txn_sent_as_association(
            &miner_account,
            sequence,
            50_000_000,
            net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            net,
        ));
        txns.push(txn.as_signed_user_txn()?.clone());
        sequence += 1;
        for (send_sequence, _j) in (0..trans_num).enumerate() {
            let receiver = Account::new();
            let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
                &receiver,
                sequence,
                1000,
                net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                net,
            ));
            txns.push(txn1.as_signed_user_txn()?.clone());
            sequence += 1;
            let txn1 = Transaction::UserTransaction(peer_to_peer_txn(
                &miner_account,
                &receiver,
                send_sequence as u64,
                1,
                net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                net.chain_id(),
            ));
            txns.push(txn1.as_signed_user_txn()?.clone());
        }

        let (block_template, _) =
            chain.create_block_template(*miner_info.address(), None, txns, vec![], None)?;
        let block =
            ConsensusStrategy::Dummy.create_block(block_template, net.time_service().as_ref())?;
        if block.transactions().len() as u64 <= trans_num {
            println!("trans {}", block.transactions().len());
        }
        let block_hash = block.header.id();
        chain.apply_with_verifier::<BasicVerifier>(block)?;

        let startup_info = StartupInfo::new(block_hash);
        storage.save_startup_info(startup_info)?;
    }
    Ok(())
}

pub fn execute_transaction_with_miner_create_account(
    storage: Arc<Storage>,
    chain: &mut BlockChain,
    net: &ChainNetwork,
    block_num: u64,
    trans_num: u64,
) -> anyhow::Result<()> {
    let miner_account = Account::new();
    let miner_info = AccountInfo::from(&miner_account);
    let mut send_sequence = 0u64;
    let (block_template, _) =
        chain.create_block_template(*miner_info.address(), None, vec![], vec![], None)?;
    let block =
        ConsensusStrategy::Dummy.create_block(block_template, net.time_service().as_ref())?;
    let block_hash = block.header.id();
    chain.apply_with_verifier::<BasicVerifier>(block)?;
    let startup_info = StartupInfo::new(block_hash);
    storage.save_startup_info(startup_info)?;
    for _i in 0..block_num {
        let mut sequence = send_sequence;
        let mut txns = vec![];
        for _j in 0..trans_num {
            let receiver = Account::new();
            let txn1 = Transaction::UserTransaction(peer_to_peer_txn(
                &miner_account,
                &receiver,
                sequence,
                1,
                net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                net.chain_id(),
            ));
            txns.push(txn1.as_signed_user_txn()?.clone());
            sequence += 1;
        }

        let (block_template, _) =
            chain.create_block_template(*miner_info.address(), None, txns, vec![], None)?;
        let block =
            ConsensusStrategy::Dummy.create_block(block_template, net.time_service().as_ref())?;
        if block.transactions().len() as u64 <= trans_num {
            println!("trans {}", block.transactions().len());
        }
        send_sequence += block.transactions().len() as u64;
        let block_hash = block.header.id();
        chain.apply_with_verifier::<BasicVerifier>(block)?;

        let startup_info = StartupInfo::new(block_hash);
        storage.save_startup_info(startup_info)?;
    }
    Ok(())
}

pub fn execute_empty_transaction_with_miner(
    storage: Arc<Storage>,
    chain: &mut BlockChain,
    net: &ChainNetwork,
    block_num: u64,
    trans_num: u64,
) -> anyhow::Result<()> {
    let miner_account = Account::new();
    let miner_info = AccountInfo::from(&miner_account);
    let mut send_sequence = 0u64;
    let (block_template, _) =
        chain.create_block_template(*miner_info.address(), None, vec![], vec![], None)?;
    let block =
        ConsensusStrategy::Dummy.create_block(block_template, net.time_service().as_ref())?;
    let block_hash = block.header.id();
    chain.apply_with_verifier::<BasicVerifier>(block)?;
    let startup_info = StartupInfo::new(block_hash);
    storage.save_startup_info(startup_info)?;
    for _i in 0..block_num {
        let mut sequence = send_sequence;
        let mut txns = vec![];
        for _j in 0..trans_num {
            let txn = build_signed_empty_txn(
                *miner_account.address(),
                miner_account.private_key(),
                sequence,
                net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                net.chain_id(),
            );
            txns.push(txn);
            sequence += 1;
        }

        let (block_template, _) =
            chain.create_block_template(*miner_info.address(), None, txns, vec![], None)?;
        let block =
            ConsensusStrategy::Dummy.create_block(block_template, net.time_service().as_ref())?;
        if block.transactions().len() as u64 <= trans_num {
            println!("trans {}", block.transactions().len());
        }
        send_sequence += block.transactions().len() as u64;
        let block_hash = block.header.id();
        chain.apply_with_verifier::<BasicVerifier>(block)?;

        let startup_info = StartupInfo::new(block_hash);
        storage.save_startup_info(startup_info)?;
    }
    Ok(())
}

pub fn execute_transaction_with_fixed_account(
    storage: Arc<Storage>,
    chain: &mut BlockChain,
    net: &ChainNetwork,
    block_num: u64,
    trans_num: u64,
) -> anyhow::Result<()> {
    let miner_account = Account::new();
    let miner_info = AccountInfo::from(&miner_account);
    let mut send_sequence = 0u64;
    let receiver = Account::new();
    let (block_template, _) =
        chain.create_block_template(*miner_info.address(), None, vec![], vec![], None)?;
    let block =
        ConsensusStrategy::Dummy.create_block(block_template, net.time_service().as_ref())?;
    let block_hash = block.header.id();
    chain.apply_with_verifier::<BasicVerifier>(block)?;
    let startup_info = StartupInfo::new(block_hash);
    storage.save_startup_info(startup_info)?;
    for _i in 0..block_num {
        let mut sequence = send_sequence;
        let mut txns = vec![];
        for _j in 0..trans_num {
            let txn1 = Transaction::UserTransaction(peer_to_peer_txn(
                &miner_account,
                &receiver,
                sequence,
                1,
                net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                net.chain_id(),
            ));
            txns.push(txn1.as_signed_user_txn()?.clone());
            sequence += 1;
        }

        let (block_template, _) =
            chain.create_block_template(*miner_info.address(), None, txns, vec![], None)?;
        let block =
            ConsensusStrategy::Dummy.create_block(block_template, net.time_service().as_ref())?;
        if block.transactions().len() as u64 <= trans_num {
            println!("trans {}", block.transactions().len());
        }
        send_sequence += block.transactions().len() as u64;
        let block_hash = block.header.id();
        chain.apply_with_verifier::<BasicVerifier>(block)?;

        let startup_info = StartupInfo::new(block_hash);
        storage.save_startup_info(startup_info)?;
    }
    Ok(())
}

// ./starcoin_db_exporter gen-block-transactions -b 1 -o ~/test > log 2>&1
// gen one miner_account and 2 * trans_num txn
// trans_num 1024, block_num 1000
pub fn execute_turbo_stm_transaction_with_fixed_account(
    storage: Arc<Storage>,
    chain: &mut BlockChain,
    net: &ChainNetwork,
    block_num: u64,
) -> anyhow::Result<()> {
    let miner_account = Account::new();
    let miner_info = AccountInfo::from(&miner_account);
    let mut sequence = 0u64;
    let mut receivers = vec![];
    let trans_num = 512;
    let mut seq = 0;
    for _i in 0..4 {
        let mut txns = vec![];
        for _j in 0..trans_num {
            let receiver1 = Account::new();
            let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
                &receiver1,
                seq,
                200_000_000,
                net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                net,
            ));
            seq += 1;
            let receiver2 = Account::new();
            let txn2 = Transaction::UserTransaction(create_account_txn_sent_as_association(
                &receiver2,
                seq,
                200_000_000,
                net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                net,
            ));
            seq += 1;
            receivers.push(receiver1);
            receivers.push(receiver2);
            txns.push(txn1.as_signed_user_txn()?.clone());
            txns.push(txn2.as_signed_user_txn()?.clone());
        }

        let (block_template, _) =
            chain.create_block_template(*miner_info.address(), None, txns, vec![], None)?;
        let block =
            ConsensusStrategy::Dummy.create_block(block_template, net.time_service().as_ref())?;
        println!("create account trans {}", block.transactions().len());
        let block_hash = block.header.id();
        chain.apply_with_verifier::<BasicVerifier>(block)?;
        let startup_info = StartupInfo::new(block_hash);
        storage.save_startup_info(startup_info)?;
        println!("receivers finish");
    }

    for _i in 0..block_num {
        let mut txns = vec![];
        for j in 0..(trans_num * 4) {
            let idx = j as usize;
            let txn1 = Transaction::UserTransaction(peer_to_peer_txn(
                receivers.get(idx).unwrap(),
                receivers.get(2 * idx).unwrap(),
                sequence,
                1000,
                net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                net.chain_id(),
            ));
            txns.push(txn1.as_signed_user_txn()?.clone());
        }
        sequence += 1;
        let (block_template, _) =
            chain.create_block_template(*miner_info.address(), None, txns, vec![], None)?;
        let block =
            ConsensusStrategy::Dummy.create_block(block_template, net.time_service().as_ref())?;
        println!("p2p trans {}", block.transactions().len());
        let block_hash = block.header.id();
        chain.apply_with_verifier::<BasicVerifier>(block)?;

        let startup_info = StartupInfo::new(block_hash);
        storage.save_startup_info(startup_info)?;
    }
    Ok(())
}

fn handle_block_cf<T>(file: &mut File, blocks: Vec<Option<T>>, ids: &[HashValue]) -> Result<()>
where
    T: serde::Serialize,
{
    for (i, block) in blocks.into_iter().enumerate() {
        let block =
            block.ok_or_else(|| format_err!("get block by hash {} error", ids.get(i).unwrap()))?;
        writeln!(file, "{}", serde_json::to_string(&block)?)?;
    }
    Ok(())
}

fn export_column(
    storage: Arc<Storage>,
    accumulator: MerkleAccumulator,
    output: PathBuf,
    column: ColumnFamilyName,
    start_num: u64,
    num: u64,
    bar: ProgressBar,
) -> Result<()> {
    // start_num > 1 increment export
    let mut file = if start_num > 1 {
        OpenOptions::new().append(true).open(output.join(column))?
    } else {
        File::create(output.join(column))?
    };
    let mut index = 1;
    let mut start_index = 0;
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
    );
    while start_index < num {
        let max_size = if start_index + BATCH_SIZE <= num {
            BATCH_SIZE
        } else {
            num - start_index
        };

        match column {
            BLOCK_ACCUMULATOR_NODE_PREFIX_NAME | TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME => {
                let ids = accumulator.get_leaves(start_index + start_num, false, max_size)?;
                for hash in ids {
                    writeln!(file, "{}", hash)?;
                }
            }
            BLOCK_PREFIX_NAME => {
                // will cache ids
                let ids = accumulator.get_leaves(start_index + start_num, false, max_size)?;
                let blocks = storage.get_blocks(ids.clone())?;
                handle_block_cf(&mut file, blocks, &ids)?;
            }
            BLOCK_INFO_PREFIX_NAME => {
                // will cache ids
                let ids = accumulator.get_leaves(start_index + start_num, false, max_size)?;
                let block_infos = storage.get_block_infos(ids.clone())?;
                handle_block_cf(&mut file, block_infos, &ids)?;
            }
            _ => {
                println!("{} not process", column);
                std::process::exit(1);
            }
        };
        start_index += max_size;
        bar.set_message(format!("export {} {}", column, index));
        bar.inc(1);
        index += 1;
    }
    file.flush()?;
    bar.finish();
    Ok(())
}

/// manifest.csv layout
/// block_accumulator num accumulator_root_hash
/// block num block.header.hash
/// block_info num block.header.hash
/// txn_accumulator num accumulator_root_hash
/// state  num state_root_hash
/// state_node_prev num state_root_hash

pub fn export_snapshot(
    from_dir: PathBuf,
    output: PathBuf,
    network: BuiltinNetworkID,
    increment: Option<bool>,
    special_block_num: Option<BlockNumber>,
) -> anyhow::Result<()> {
    let start_time = SystemTime::now();
    let net = ChainNetwork::new_builtin(network);
    let db_storage = DBStorage::open_with_cfs(
        from_dir.join("starcoindb/db/starcoindb"),
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (chain_info, _) =
        Genesis::init_and_check_storage(&net, storage.clone(), from_dir.as_ref())?;
    let chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage.clone(),
        None,
    )
    .expect("create block chain should success.");
    let block_num = chain.status().head().number();
    let mut cur_num = if block_num <= SNAP_GAP {
        block_num
    } else {
        block_num - SNAP_GAP
    };
    // For debug purpose
    if let Some(special_num) = special_block_num {
        if special_num <= cur_num {
            cur_num = special_num;
        }
    }
    let cur_block = chain
        .get_block_by_number(cur_num)?
        .ok_or_else(|| format_err!("get block by number {} error", cur_num))?;
    let chain = BlockChain::new(net.time_service(), cur_block.id(), storage.clone(), None)
        .expect("create block chain should success.");

    let cur_num = chain.epoch().start_block_number();

    // For fork block verifier the parent block, So need block number sub 1
    let cur_num_prev = cur_num - 1;

    // increment export read num
    let inc_export = increment.unwrap_or(false);
    let mut old_snapshot_nums: HashMap<String, u64> = HashMap::new();
    if inc_export {
        let reader = BufReader::new(File::open(output.join("manifest.csv"))?);
        for record in reader.lines() {
            let record = record?;
            let str_list: Vec<&str> = record.split(' ').collect();
            if str_list.len() != 3 {
                println!("manifest.csv {} error", record);
                std::process::exit(1);
            }
            let column = str_list[0].to_string();
            let num = str_list[1].parse::<u64>()?;
            old_snapshot_nums.insert(column, num);
        }
        if old_snapshot_nums.len() != 6 {
            println!("increment export snapshot manifest.cvs error");
            std::process::exit(1);
        }
        let old_block_num = *old_snapshot_nums.get(BLOCK_PREFIX_NAME).ok_or_else(|| {
            format_err!(
                "increment export snapshot get {} number error",
                BLOCK_PREFIX_NAME
            )
        })?;
        if old_block_num + BLOCK_GAP >= cur_num {
            println!("increment snapshot gap too small");
            return Ok(());
        }
        println!(
            "chain height {} snapshot block cur height {} old height {}",
            chain_info.head().number(),
            cur_num,
            old_block_num
        );
    } else {
        println!(
            "chain height {} snapshot block height {}",
            chain_info.head().number(),
            cur_num
        );
    }

    let block = chain
        .get_block_by_number(cur_num)?
        .ok_or_else(|| format_err!("get block by number {} error", cur_num))?;
    let block_info = chain
        .get_block_info(Some(block.id()))?
        .ok_or_else(|| format_err!("get block info by hash {} error", block.id()))?;
    let block_accumulator_info = block_info.get_block_accumulator_info();
    let mut manifest_list = vec![];
    let mut handles = Vec::with_capacity(5);

    manifest_list.push((
        BLOCK_ACCUMULATOR_NODE_PREFIX_NAME,
        cur_num,
        block_accumulator_info.accumulator_root,
    ));
    manifest_list.push((BLOCK_PREFIX_NAME, cur_num, block.header.id()));
    manifest_list.push((BLOCK_INFO_PREFIX_NAME, cur_num, block.header.id()));
    let txn_accumulator_info = block_info.get_txn_accumulator_info();
    manifest_list.push((
        TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME,
        txn_accumulator_info.get_num_leaves() - 1,
        txn_accumulator_info.accumulator_root,
    ));
    let mbar = MultiProgress::new();
    for (column, num_record, _hash) in manifest_list.clone() {
        let accumulator = match column {
            BLOCK_ACCUMULATOR_NODE_PREFIX_NAME | BLOCK_PREFIX_NAME | BLOCK_INFO_PREFIX_NAME => {
                MerkleAccumulator::new_with_info(
                    block_accumulator_info.clone(),
                    storage.get_accumulator_store(AccumulatorStoreType::Block),
                )
            }
            TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME => MerkleAccumulator::new_with_info(
                txn_accumulator_info.clone(),
                storage.get_accumulator_store(AccumulatorStoreType::Transaction),
            ),
            _ => {
                println!("{} not process", column);
                std::process::exit(1);
            }
        };
        let old_start_num = *old_snapshot_nums.get(column).unwrap_or(&0);
        let num = num_record - old_start_num;
        let start_num = old_start_num + 1;
        let storage2 = storage.clone();
        let output2 = output.clone();
        let bar = mbar.add(ProgressBar::new(num / BATCH_SIZE));
        let handle = thread::spawn(move || {
            export_column(storage2, accumulator, output2, column, start_num, num, bar)
        });
        handles.push(handle);
    }

    let nums = Arc::new(AtomicU64::default());
    let block = chain
        .get_block_by_number(cur_num)?
        .ok_or_else(|| format_err!("get block by number {} error", cur_num))?;
    let state_root = block.header.state_root();
    handles.push(chain_snapshot(
        block,
        storage.clone(),
        &mbar,
        output.clone(),
        nums.clone(),
        STATE_NODE_PREFIX_NAME,
    )?);

    let nums_prev = Arc::new(AtomicU64::default());
    let block = chain
        .get_block_by_number(cur_num_prev)?
        .ok_or_else(|| format_err!("get block by number {} error", cur_num_prev))?;
    let state_root_prev = block.header.state_root();
    handles.push(chain_snapshot(
        block,
        storage,
        &mbar,
        output.clone(),
        nums_prev.clone(),
        STATE_NODE_PREFIX_NAME_PREV,
    )?);

    mbar.join_and_clear()?;
    for handle in handles {
        handle.join().unwrap().unwrap();
    }

    manifest_list.push((
        STATE_NODE_PREFIX_NAME,
        nums.load(Ordering::Relaxed),
        state_root,
    ));

    println!(
        "{} nums {}",
        STATE_NODE_PREFIX_NAME,
        nums.load(Ordering::Relaxed)
    );

    manifest_list.push((
        STATE_NODE_PREFIX_NAME_PREV,
        nums_prev.load(Ordering::Relaxed),
        state_root_prev,
    ));

    println!(
        "{} nums {}",
        STATE_NODE_PREFIX_NAME_PREV,
        nums_prev.load(Ordering::Relaxed)
    );

    // save manifest
    let name_manifest = "manifest.csv".to_string();
    let mut file_manifest = File::create(output.join(name_manifest))?;
    for (path, num, hash) in manifest_list {
        writeln!(file_manifest, "{} {} {}", path, num, hash)?;
    }
    file_manifest.flush()?;

    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("export snapshot use time: {:?}", use_time.as_secs());
    Ok(())
}

fn chain_snapshot(
    block: Block,
    storage: Arc<Storage>,
    mbar: &MultiProgress,
    output: PathBuf,
    nums: Arc<AtomicU64>,
    name: &'static str,
) -> Result<JoinHandle<Result<(), anyhow::Error>>, anyhow::Error> {
    // get state
    let state_root = block.header.state_root();
    let statedb = ChainStateDB::new(storage, Some(state_root));
    let bar = mbar.add(ProgressBar::new(20000000 / BATCH_SIZE));
    let state_handler = thread::spawn(move || {
        let mut index = 1;
        let mut file = File::create(output.join(name))?;
        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
        );
        let global_states_iter = statedb.dump_iter()?;
        for (account_address, account_state_set) in global_states_iter {
            writeln!(
                file,
                "{} {}",
                serde_json::to_string(&account_address)?,
                serde_json::to_string(&account_state_set)?
            )?;

            if index % BATCH_SIZE == 0 {
                bar.set_message(format!("export state {}", index / BATCH_SIZE));
                bar.inc(1);
            }
            index += 1;
        }
        file.flush()?;
        bar.finish();
        nums.store(index - 1, Ordering::Relaxed);
        Ok(())
    });

    Ok(state_handler)
}

fn import_column(
    storage: Arc<Storage>,
    accumulator: MerkleAccumulator,
    input_path: PathBuf,
    column: String,
    verify_hash: HashValue,
    bar: ProgressBar,
) -> Result<()> {
    let reader = BufReader::new(File::open(input_path.join(column.clone()))?);
    let mut index = 1;
    let mut leaves = vec![];
    let mut block_hash = HashValue::zero();
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
    );
    for line in reader.lines() {
        let line = line?;
        match column.as_str() {
            BLOCK_ACCUMULATOR_NODE_PREFIX_NAME | TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME => {
                leaves.push(HashValue::from_hex_literal(line.as_str())?);
                if index % BATCH_SIZE == 0 {
                    accumulator.append(&leaves)?;
                    accumulator.flush()?;
                    leaves.clear();
                }
            }
            BLOCK_PREFIX_NAME => {
                let block: Block = serde_json::from_str(line.as_str())?;
                block_hash = block.id();
                storage.commit_block(block)?;
            }
            BLOCK_INFO_PREFIX_NAME => {
                let block_info: BlockInfo = serde_json::from_str(line.as_str())?;
                block_hash = block_info.block_id;
                storage.save_block_info(block_info)?;
            }
            _ => {
                println!("{} not process", column);
                std::process::exit(1);
            }
        }
        if index % BATCH_SIZE == 0 {
            bar.set_message(format!("import {} {}", column, index / BATCH_SIZE));
            bar.inc(1);
        }
        index += 1;
    }
    match column.as_str() {
        BLOCK_ACCUMULATOR_NODE_PREFIX_NAME | TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME => {
            if !leaves.is_empty() {
                accumulator.append(&leaves)?;
                accumulator.flush()?;
            }
            if accumulator.root_hash() == verify_hash {
                println!("{} hash match", column);
            } else {
                println!(
                    "{} hash not match root_hash {} verify_hash {}",
                    column,
                    accumulator.root_hash(),
                    verify_hash
                );
                std::process::exit(1);
            }
        }
        BLOCK_PREFIX_NAME | BLOCK_INFO_PREFIX_NAME => {
            if verify_hash == block_hash {
                println!("{} hash match", column);
            } else {
                println!(
                    "{} hash not match block_hash {} verify_hash {}",
                    column, block_hash, verify_hash
                );
                std::process::exit(1);
            }
        }
        _ => {
            println!("{} not process", column);
            std::process::exit(1);
        }
    }
    bar.finish();
    Ok(())
}

pub fn apply_snapshot(
    to_dir: PathBuf,
    input_path: PathBuf,
    network: BuiltinNetworkID,
) -> anyhow::Result<()> {
    let start_time = SystemTime::now();
    let net = ChainNetwork::new_builtin(network);
    let db_storage = DBStorage::new(to_dir.join("starcoindb/db"), RocksdbConfig::default(), None)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);

    let (chain_info, _) = Genesis::init_and_check_storage(&net, storage.clone(), to_dir.as_ref())?;
    let chain = Arc::new(std::sync::Mutex::new(
        BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage.clone(),
            None,
        )
        .expect("create block chain should success."),
    ));

    let mut block_hash = HashValue::zero();
    let mut block_num = 1;
    let mut handles = vec![];
    let reader = BufReader::new(File::open(input_path.join("manifest.csv"))?);
    let mut file_list = vec![];
    for record in reader.lines() {
        let record = record?;
        let str_list: Vec<&str> = record.split(' ').collect();
        if str_list.len() != 3 {
            println!("manifest.csv {} error", record);
            std::process::exit(1);
        }
        let column = str_list[0].to_string();
        let nums = str_list[1].parse::<BlockNumber>()?;
        let verify_hash = HashValue::from_hex_literal(str_list[2])?;
        if str_list[0] == BLOCK_PREFIX_NAME {
            block_hash = verify_hash;
            block_num = nums;
        }
        file_list.push((column, nums, verify_hash));
    }

    for (file_name, nums, _) in file_list.iter() {
        let file = BufReader::new(File::open(input_path.join(file_name))?);
        let cnt = file.lines().count();
        if cnt as BlockNumber != *nums {
            println!("file {} line nums {} not equal {}", file_name, cnt, *nums);
            std::process::exit(2);
        }
    }

    let mbar = MultiProgress::new();
    for item in file_list.iter().take(file_list.len() - 3) {
        let (column, nums, verify_hash) = item.clone();
        let storage2 = storage.clone();
        let accumulator = match column.as_str() {
            BLOCK_ACCUMULATOR_NODE_PREFIX_NAME | BLOCK_PREFIX_NAME | BLOCK_INFO_PREFIX_NAME => {
                MerkleAccumulator::new_with_info(
                    chain.lock().unwrap().status().info.block_accumulator_info,
                    storage.get_accumulator_store(AccumulatorStoreType::Block),
                )
            }
            TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME => MerkleAccumulator::new_with_info(
                chain.lock().unwrap().status().info.txn_accumulator_info,
                storage.get_accumulator_store(AccumulatorStoreType::Transaction),
            ),
            _ => {
                println!("{} not process", column);
                std::process::exit(1);
            }
        };
        let input_path2 = input_path.clone();
        let bar = mbar.add(ProgressBar::new(nums / BATCH_SIZE));
        let handle = thread::spawn(move || {
            import_column(storage2, accumulator, input_path2, column, verify_hash, bar)
        });
        handles.push(handle);
    }

    // STATE_NODE_PREFIX_NAME
    if let Some((column, nums, verify_hash)) = file_list.get(file_list.len() - 2).cloned() {
        let bar = mbar.add(ProgressBar::new(nums / BATCH_SIZE));
        let input_path = input_path.clone();
        let chain = chain.clone();
        let handle = thread::spawn(move || {
            let reader = BufReader::new(File::open(input_path.join(column))?);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
            );
            let mut index = 1;
            let mut account_states = vec![];
            for line in reader.lines() {
                let line = line?;
                let strs: Vec<&str> = line.split(' ').collect();
                let account_address: AccountAddress = serde_json::from_str(strs[0])?;
                let account_state_set: AccountStateSet = serde_json::from_str(strs[1])?;
                account_states.push((account_address, account_state_set));
                index += 1;
                if index % BATCH_SIZE == 0 {
                    bar.set_message(format!(
                        "import {} index {}",
                        STATE_NODE_PREFIX_NAME,
                        index / BATCH_SIZE
                    ));
                    bar.inc(1);
                }
            }
            bar.finish();
            let chain_state_set = ChainStateSet::new(account_states);
            let mut chain = chain.lock().unwrap();
            chain.chain_state().apply(chain_state_set)?;
            if chain.chain_state_reader().state_root() == verify_hash {
                println!("snapshot_state hash match");
            } else {
                println!(
                    "snapshot_state hash not match state_root {} verify_hash {}",
                    chain.chain_state_reader().state_root(),
                    verify_hash
                );
                std::process::exit(1);
            }
            Ok(())
        });
        handles.push(handle);
    }

    // STATE_NODE_PREFIX_NAME_PREV
    if let Some((column, nums, verify_hash)) = file_list.last().cloned() {
        let bar = mbar.add(ProgressBar::new(nums / BATCH_SIZE));
        let handle = thread::spawn(move || {
            let reader = BufReader::new(File::open(input_path.join(column))?);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
            );
            let mut index = 1;
            let mut account_states = vec![];
            for line in reader.lines() {
                let line = line?;
                let strs: Vec<&str> = line.split(' ').collect();
                let account_address: AccountAddress = serde_json::from_str(strs[0])?;
                let account_state_set: AccountStateSet = serde_json::from_str(strs[1])?;
                account_states.push((account_address, account_state_set));
                index += 1;
                if index % BATCH_SIZE == 0 {
                    bar.set_message(format!(
                        "import {} index {}",
                        STATE_NODE_PREFIX_NAME_PREV,
                        index / BATCH_SIZE
                    ));
                    bar.inc(1);
                }
            }
            bar.finish();
            let chain_state_set = ChainStateSet::new(account_states);
            let mut chain = chain.lock().unwrap();
            chain.chain_state().apply(chain_state_set)?;
            if chain.chain_state_reader().state_root() == verify_hash {
                println!("snapshot_state hash match");
            } else {
                println!(
                    "snapshot_state hash not match state_root {} verify_hash {}",
                    chain.chain_state_reader().state_root(),
                    verify_hash
                );
                std::process::exit(1);
            }
            Ok(())
        });
        handles.push(handle);
    }

    mbar.join_and_clear()?;
    for handle in handles {
        handle.join().unwrap().unwrap();
    }

    // save startup_info
    let startup_info = StartupInfo::new(block_hash);
    storage.save_startup_info(startup_info)?;
    // save import snapshot range
    let snapshot_range = SnapshotRange::new(1, block_num);
    storage.save_snapshot_range(snapshot_range)?;
    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("apply snapshot use time: {:?}", use_time.as_secs());
    Ok(())
}

#[derive(Serialize, Debug)]
pub struct AccountData<R: Serialize> {
    address: AccountAddress,
    #[serde(flatten)]
    resource: Option<R>,
}

pub fn export_resource(
    db: &str,
    output: &Path,
    block_hash: HashValue,
    resource_struct_tag: StructTag,
    fields: &[String],
) -> anyhow::Result<()> {
    let db_storage = DBStorage::open_with_cfs(
        db,
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    let storage = Storage::new(StorageInstance::new_db_instance(db_storage))?;
    let storage = Arc::new(storage);
    let block = storage
        .get_block(block_hash)?
        .ok_or_else(|| anyhow::anyhow!("block {} not exist", block_hash))?;

    let root = block.header.state_root();
    let statedb = ChainStateDB::new(storage, Some(root));
    let value_annotator = MoveValueAnnotator::new(&statedb);

    let mut csv_writer = csv::WriterBuilder::new().from_path(output)?;

    // write csv header.
    {
        csv_writer.write_field("address")?;
        for f in fields {
            csv_writer.write_field(f)?;
        }
        csv_writer.write_record(None::<&[u8]>)?;
    }

    use std::time::Instant;

    let now = Instant::now();

    let global_states_iter = statedb.dump_iter()?;
    println!("t1: {}", now.elapsed().as_millis());

    let now = Instant::now();
    let mut t1_sum = 0;
    let mut t2_sum = 0;
    let mut loop_count = 0;
    let mut now3 = Instant::now();
    let mut iter_time = 0;
    for (account_address, account_state_set) in global_states_iter {
        iter_time += now3.elapsed().as_nanos();

        loop_count += 1;

        let now1 = Instant::now();
        let resource_set = account_state_set.resource_set().unwrap();
        // t1_sum += now1.elapsed().as_micros();
        t1_sum += now1.elapsed().as_nanos();
        // println!("t1 sum in loop: {}", t1_sum);

        let now2 = Instant::now();
        for (k, v) in resource_set.iter() {
            let struct_tag = StructTag::decode(k.as_slice())?;
            if struct_tag == resource_struct_tag {
                let annotated_struct =
                    value_annotator.view_struct(resource_struct_tag.clone(), v.as_slice())?;
                let resource_struct = annotated_struct;
                let resource_json_value = serde_json::to_value(MoveStruct(resource_struct))?;
                let resource = Some(resource_json_value);
                let record: Option<Vec<_>> = resource
                    .as_ref()
                    .map(|v| fields.iter().map(|f| v.pointer(f.as_str())).collect());
                if let Some(mut record) = record {
                    let account_value = serde_json::to_value(account_address).unwrap();
                    record.insert(0, Some(&account_value));
                    csv_writer.serialize(record)?;
                }
                break;
            }
        }
        // let d = now2.elapsed().as_micros();
        let d = now2.elapsed().as_nanos();
        // println!("d is: {}", d);
        t2_sum += d;
        // println!("t2 sum in loop: {}", t2_sum);
        now3 = Instant::now();
    }
    println!("iter time: {}, {}", iter_time, iter_time / 1_000_000);
    println!("loop count: {}", loop_count);
    println!("t1_sum: {}, {}", t1_sum, t1_sum / 1000000);
    println!("t2_sum: {}, {}", t2_sum, t2_sum / 1000000);

    println!("t2: {}", now.elapsed().as_millis());
    csv_writer.flush()?;
    Ok(())
}

#[derive(Debug, Clone)]
struct MoveStruct(AnnotatedMoveStruct);

impl serde::Serialize for MoveStruct {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.value.len()))?;
        for (field, value) in &self.0.value {
            map.serialize_entry(field.as_str(), &MoveValue(value.clone()))?;
        }
        map.end()
    }
}

#[derive(Debug, Clone)]
struct MoveValue(AnnotatedMoveValue);

impl serde::Serialize for MoveValue {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            AnnotatedMoveValue::Bool(b) => serializer.serialize_bool(*b),
            AnnotatedMoveValue::U8(v) => serializer.serialize_u8(*v),
            AnnotatedMoveValue::U64(v) => serializer.serialize_u64(*v),
            AnnotatedMoveValue::U128(v) => serializer.serialize_u128(*v),
            AnnotatedMoveValue::Address(v) => v.serialize(serializer),
            AnnotatedMoveValue::Vector(v) => {
                let vs: Vec<_> = v.clone().into_iter().map(MoveValue).collect();
                vs.serialize(serializer)
            }
            AnnotatedMoveValue::Bytes(v) => hex::encode(v).serialize(serializer),
            AnnotatedMoveValue::Struct(v) => MoveStruct(v.clone()).serialize(serializer),
            _ => todo!("XXX FXIME YSG"),
        }
    }
}

pub fn gen_turbo_stm_transactions(to_dir: PathBuf, block_num: Option<u64>) -> anyhow::Result<()> {
    starcoin_logger::init();
    let net = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let db_storage = DBStorage::new(to_dir.join("starcoindb/db"), RocksdbConfig::default(), None)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (chain_info, _) = Genesis::init_and_check_storage(&net, storage.clone(), to_dir.as_ref())?;
    let mut chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage.clone(),
        None,
    )
    .expect("create block chain should success.");
    let block_num = block_num.unwrap_or(1000);
    execute_turbo_stm_transaction_with_fixed_account(storage, &mut chain, &net, block_num)
}

pub fn apply_turbo_stm_block(
    to_dir: PathBuf,
    turbo_stm_to_dir: PathBuf,
    input_path: PathBuf,
) -> anyhow::Result<()> {
    ::starcoin_logger::init();
    let net = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let db_storage_seq =
        DBStorage::new(to_dir.join("starcoindb/db"), RocksdbConfig::default(), None)?;
    let storage_seq = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage_seq,
    ))?);
    let (chain_info_seq, _) =
        Genesis::init_and_check_storage(&net, storage_seq.clone(), to_dir.as_ref())?;
    let mut chain_seq = BlockChain::new(
        net.time_service(),
        chain_info_seq.head().id(),
        storage_seq.clone(),
        None,
    )
    .expect("create block chain should success.");
    let cur_num = chain_seq.status().head().number();

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
        println!(
            "current number {}, import [{},{}] block number",
            cur_num, start, end
        );
    }
    println!("seq execution");

    for item in blocks.iter().take(4) {
        chain_seq.apply_with_verifier::<NoneVerifier>(item.clone())?;
    }
    let mut block_hash = HashValue::zero();
    let start_time = SystemTime::now();
    for item in blocks.iter().skip(4) {
        let block = item.clone();
        block_hash = block.header().id();
        chain_seq.apply_with_verifier::<NoneVerifier>(block)?;
    }
    let startup_info = StartupInfo::new(block_hash);
    storage_seq.save_startup_info(startup_info)?;
    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("seq apply block use time: {:?}", use_time.as_secs());

    let db_storage_stm = DBStorage::new(
        turbo_stm_to_dir.join("starcoindb/db"),
        RocksdbConfig::default(),
        None,
    )?;
    let storage_stm = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage_stm,
    ))?);
    let (chain_info_stm, _) =
        Genesis::init_and_check_storage(&net, storage_stm.clone(), turbo_stm_to_dir.as_ref())?;
    let mut chain_stm = BlockChain::new(
        net.time_service(),
        chain_info_stm.head().id(),
        storage_stm.clone(),
        None,
    )
    .expect("create block chain should success.");

    println!("stm execution");
    for item in blocks.iter().take(4) {
        chain_stm.apply_with_verifier::<NoneVerifier>(item.clone())?;
    }
    let mut block_hash = HashValue::zero();
    let start_time = SystemTime::now();
    StarcoinVM::set_concurrency_level_once(num_cpus::get());
    for item in blocks.iter().skip(4) {
        let block = item.clone();
        block_hash = block.header().id();
        chain_stm.apply_with_verifier::<NoneVerifier>(block)?;
    }
    let startup_info = StartupInfo::new(block_hash);
    storage_stm.save_startup_info(startup_info)?;
    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("stm apply block use time: {:?}", use_time.as_secs());
    Ok(())
}

pub fn verify_block(
    from_dir: PathBuf,
    network: BuiltinNetworkID,
    start: BlockNumber,
    end: Option<BlockNumber>,
    verifier: Verifier,
) -> anyhow::Result<()> {
    ::starcoin_logger::init();
    let net = ChainNetwork::new_builtin(network);
    let db_storage = DBStorage::open_with_cfs(
        from_dir.join("starcoindb/db/starcoindb"),
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (chain_info, _) =
        Genesis::init_and_check_storage(&net, storage.clone(), from_dir.as_ref())?;
    let chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage.clone(),
        None,
    )
    .expect("create block chain should success.");
    let start_num = start;
    let end_num = end.unwrap_or_else(|| chain.status().head().number());
    let start_time = SystemTime::now();
    let thread_cnt = num_cpus::get() / 2;
    let avg = (end_num - start_num + 1) / (thread_cnt as u64);
    let mut handles = vec![];
    for i in 0..thread_cnt {
        let st = start_num + i as u64 * avg;
        let mut end = start_num + (i as u64 + 1) * avg - 1;
        if end > end_num {
            end = end_num;
        }
        let chain = BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage.clone(),
            None,
        )
        .expect("create block chain should success.");
        let verifier2 = verifier.clone();
        let handle = thread::spawn(move || {
            verify_block_range(chain, st, end, verifier2).expect("verify_block_range panic")
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("verify block use time: {:?}", use_time.as_secs());
    Ok(())
}

fn verify_block_range(
    chain: BlockChain,
    start_num: u64,
    end_num: u64,
    verifier: Verifier,
) -> Result<()> {
    let name = format!("file-{}-{}", start_num, end_num);
    let mut file = File::create(name)?;
    writeln!(file, "block [{}..{}]", start_num, end_num)?;
    let bar = ProgressBar::new(end_num - start_num + 1);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
    );
    let mut cnt = 0;
    for block_number in start_num..=end_num {
        let block = chain
            .get_block_by_number(block_number)?
            .ok_or_else(|| format_err!("{} get block error", block_number))?;
        let mut cur_chain = chain.fork(block.header().parent_hash())?;
        let res = match verifier {
            Verifier::Basic => cur_chain.verify_without_save::<BasicVerifier>(block),
            Verifier::Consensus => cur_chain.verify_without_save::<ConsensusVerifier>(block),
            Verifier::Full => cur_chain.verify_without_save::<FullVerifier>(block),
            Verifier::None => cur_chain.verify_without_save::<NoneVerifier>(block),
        };
        if res.is_err() {
            writeln!(file, "block {} verify error", block_number)?;
        }
        bar.set_message(format!("verify block {}", block_number));
        bar.inc(1);
        cnt += 1;
        if cnt % 1000 == 0 {
            writeln!(file, "block {} verify", start_num + cnt)?;
        }
    }
    bar.finish();
    file.flush()?;
    Ok(())
}

pub fn block_output(
    from_dir: PathBuf,
    network: BuiltinNetworkID,
    block_number: BlockNumber,
) -> anyhow::Result<()> {
    ::starcoin_logger::init();
    let net = ChainNetwork::new_builtin(network);
    let db_storage = DBStorage::open_with_cfs(
        from_dir.join("starcoindb/db/starcoindb"),
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (chain_info, _) =
        Genesis::init_and_check_storage(&net, storage.clone(), from_dir.as_ref())?;
    let chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage.clone(),
        None,
    )
    .expect("create block chain should success.");
    let block = chain
        .get_block_by_number(block_number)?
        .ok_or_else(|| format_err!("{} get block error", block_number))?;
    BlockChain::set_output_block();
    let mut chain = BlockChain::new(
        net.time_service(),
        block.header.parent_hash(),
        storage,
        None,
    )
    .expect("create block chain should success.");
    chain.verify_without_save::<BasicVerifier>(block)?;
    Ok(())
}

pub fn apply_block_output(
    to_dir: PathBuf,
    input_path: PathBuf,
    network: BuiltinNetworkID,
) -> anyhow::Result<()> {
    ::starcoin_logger::init();
    let net = ChainNetwork::new_builtin(network);
    let db_storage = DBStorage::new(to_dir.join("starcoindb/db"), RocksdbConfig::default(), None)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (_chain_info, _) = Genesis::init_and_check_storage(&net, storage.clone(), to_dir.as_ref())?;
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

    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("load blocks from file use time: {:?}", use_time.as_millis());
    let bar = ProgressBar::new(blocks.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
    );
    BlockChain::set_output_block();
    for block in blocks {
        let block_number = block.header().number();
        let mut chain = BlockChain::new(
            net.time_service(),
            block.header().parent_hash(),
            storage.clone(),
            None,
        )
        .expect("create block chain should success.");
        chain.verify_without_save::<BasicVerifier>(block)?;
        bar.set_message(format!("apply block {}", block_number));
        bar.inc(1);
    }
    bar.finish();
    Ok(())
}

fn save_startup_info(
    to_dir: PathBuf,
    network: BuiltinNetworkID,
    hash_value: HashValue,
) -> anyhow::Result<()> {
    ::starcoin_logger::init();
    let net = ChainNetwork::new_builtin(network);
    let db_storage = DBStorage::new(to_dir.join("starcoindb/db"), RocksdbConfig::default(), None)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (_chain_info, _) = Genesis::init_and_check_storage(&net, storage.clone(), to_dir.as_ref())?;
    let startup_info = StartupInfo::new(hash_value);
    storage.save_startup_info(startup_info)?;
    Ok(())
}

fn token_supply(
    from_dir: PathBuf,
    output: PathBuf,
    network: BuiltinNetworkID,
    block_number: Option<BlockNumber>,
    resource_struct_tag: StructTag,
) -> anyhow::Result<()> {
    let net = ChainNetwork::new_builtin(network);
    let db_storage = DBStorage::open_with_cfs(
        from_dir.join("starcoindb/db/starcoindb"),
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (chain_info, _) =
        Genesis::init_and_check_storage(&net, storage.clone(), from_dir.as_ref())?;
    let chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage.clone(),
        None,
    )
    .expect("create block chain should success.");
    let cur_num = block_number.unwrap_or_else(|| chain_info.head().number());
    let block = chain
        .get_block_by_number(cur_num)?
        .ok_or_else(|| format_err!("get block by number {} error", cur_num))?;

    let root = block.header.state_root();
    let statedb = ChainStateDB::new(storage.clone(), Some(root));
    let value_annotator = MoveValueAnnotator::new(&statedb);

    let state_tree = StateTree::<AccountAddress>::new(storage.clone(), Some(root));

    let mut file = File::create(output)?;

    let global_states = state_tree.dump()?;

    use std::time::Instant;
    let now = Instant::now();
    let mut sum: u128 = 0;
    for (address_bytes, account_state_bytes) in global_states.iter() {
        let account: AccountAddress = bcs_ext::from_bytes(address_bytes)?;
        let account_state: AccountState = account_state_bytes.as_slice().try_into()?;
        let resource_root = account_state.storage_roots()[DataType::RESOURCE.storage_index()];
        let resource = match resource_root {
            None => None,
            Some(root) => {
                let account_tree = StateTree::<StructTag>::new(storage.clone(), Some(root));
                let data = account_tree.get(&resource_struct_tag)?;

                if let Some(d) = data {
                    let annotated_struct =
                        value_annotator.view_struct(resource_struct_tag.clone(), d.as_slice())?;
                    let resource = annotated_struct;
                    let resource_json_value = serde_json::to_value(MoveStruct(resource))?;
                    Some(resource_json_value)
                } else {
                    None
                }
            }
        };
        if let Some(res) = resource {
            let balance = (res
                .get("token")
                .unwrap()
                .get("value")
                .unwrap()
                .as_f64()
                .unwrap()
                / 1000000000.0) as u128;
            if balance > 0 {
                writeln!(file, "{} {}", account, balance)?;
                sum += balance;
            }
        }
    }
    println!("t2: {}", now.elapsed().as_millis());
    writeln!(file, "total {}", sum)?;
    writeln!(file, "cur height {}", cur_num)?;
    file.flush()?;
    Ok(())
}
