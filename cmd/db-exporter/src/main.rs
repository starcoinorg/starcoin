// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use bcs_ext::Sample;
use csv::Writer;
use indicatif::{ProgressBar, ProgressStyle};
use starcoin_account_api::AccountInfo;
use starcoin_accumulator::Accumulator;
use starcoin_chain::verifier::{
    BasicVerifier, ConsensusVerifier, FullVerifier, NoneVerifier, Verifier,
};
use starcoin_chain::{BlockChain, ChainReader, ChainWriter};
use starcoin_config::{BuiltinNetworkID, ChainNetwork, RocksdbConfig};
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_executor::account::{create_account_txn_sent_as_association, peer_to_peer_txn};
use starcoin_executor::DEFAULT_EXPIRATION_TIME;
use starcoin_genesis::Genesis;
//use starcoin_state_tree::{StateNodeStore, StateTree};
use starcoin_storage::block::FailedBlock;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::ValueCodec;
use starcoin_storage::storage::{InnerStore, StorageInstance};
use starcoin_storage::{
    BlockStore, Storage, StorageVersion, BLOCK_ACCUMULATOR_NODE_PREFIX_NAME,
    BLOCK_HEADER_PREFIX_NAME, BLOCK_PREFIX_NAME, FAILED_BLOCK_PREFIX_NAME, STATE_NODE_PREFIX_NAME,
    TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME,
};
use starcoin_transaction_builder::build_signed_empty_txn;
use starcoin_types::account::Account;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::{Block, BlockHeader, BlockNumber};
use starcoin_types::startup_info::StartupInfo;
use starcoin_types::transaction::Transaction;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::ops::Add;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;
use structopt::StructOpt;
//use forkable_jellyfish_merkle::node_type::LeafNode;
use starcoin_statedb::ChainStateDB;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet, StateSet};
use starcoin_statedb::ChainStateReader;
use starcoin_statedb::ChainStateWriter;

const BLOCK_GAP: u64 = 1000;
const BACK_SIZE: u64 = 10000;
const SNAP_GAP: u64 = 128;

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

#[derive(StructOpt)]
#[structopt(version = "0.1.0", author = "Starcoin Core Dev <dev@starcoin.org>")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(StructOpt)]
enum Cmd {
    Exporter(ExporterOptions),
    Checkkey(CheckKeyOptions),
    ExportBlockRange(ExportBlockRangeOptions),
    ApplyBlock(ApplyBlockOptions),
    StartupInfoBack(StartupInfoBackOptions),
    GenBlockTransactions(GenBlockTransactionsOptions),
    ExportSnapshot(ExportSnapshotOptions),
    ApplySnapshot(ApplySnapshotOptions),
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "db-exporter", about = "starcoin db exporter")]
pub struct ExporterOptions {
    #[structopt(long, short = "o", parse(from_os_str))]
    /// output file, like accounts.csv, default is stdout.
    pub output: Option<PathBuf>,
    #[structopt(long, short = "i", parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/barnard/starcoindb/db/starcoindb
    pub db_path: PathBuf,

    #[structopt(long, short = "s")]
    /// the table of database which to export, block,block_header
    pub schema: DbSchema,
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "checkkey", about = "starcoin db check key")]
pub struct CheckKeyOptions {
    #[structopt(long, short = "i", parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/barnard/starcoindb/db/starcoindb
    pub db_path: PathBuf,
    #[structopt(long, short = "n",
    possible_values=&["block", "block_header"],)]
    pub cf_name: String,
    #[structopt(long, short = "b")]
    pub block_hash: HashValue,
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "export-block-range", about = "export block range")]
pub struct ExportBlockRangeOptions {
    #[structopt(long, short = "n")]
    /// Chain Network, like main, proxima
    pub net: BuiltinNetworkID,
    #[structopt(long, short = "o", parse(from_os_str))]
    /// output dir, like ~/, output filename like ~/block_start_end.csv
    pub output: PathBuf,
    #[structopt(long, short = "i", parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub db_path: PathBuf,
    #[structopt(long, short = "s")]
    pub start: BlockNumber,
    #[structopt(long, short = "e")]
    pub end: BlockNumber,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "apply-block-range", about = "apply block range")]
pub struct ApplyBlockOptions {
    #[structopt(long, short = "n")]
    /// Chain Network
    pub net: BuiltinNetworkID,
    #[structopt(long, short = "o", parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub to_path: PathBuf,
    #[structopt(long, short = "i", parse(from_os_str))]
    /// input file, like ~/block_start_end.csv
    pub input_path: PathBuf,
    #[structopt(possible_values = &Verifier::variants(), case_insensitive = true)]
    /// Verify type:  Basic, Consensus, Full, None, eg.
    pub verifier: Option<Verifier>,
    #[structopt(long, short = "w")]
    /// Watch metrics logs.
    pub watch: bool,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "startup_info_back", about = "startup info back")]
pub struct StartupInfoBackOptions {
    #[structopt(long, short = "n")]
    /// Chain Network
    pub net: BuiltinNetworkID,
    #[structopt(long, short = "o", parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub to_path: PathBuf,
    /// startupinfo BlockNumber back off size
    #[structopt(long, short = "b")]
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

#[derive(Debug, StructOpt)]
#[structopt(name = "gen_block_transactions", about = "gen block transactions")]
pub struct GenBlockTransactionsOptions {
    #[structopt(long, short = "o", parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/halley
    pub to_path: PathBuf,
    #[structopt(long, short = "b")]
    pub block_num: Option<u64>,
    #[structopt(long, short = "t")]
    pub trans_num: Option<u64>,
    #[structopt(long, short = "p", possible_values=&["CreateAccount", "FixAccount", "EmptyTxn"],)]
    /// txn type
    pub txn_type: Txntype,
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "export-snapshot", about = "export snapshot")]
pub struct ExportSnapshotOptions {
    #[structopt(long, short = "n")]
    /// Chain Network, like main, proxima
    pub net: BuiltinNetworkID,
    #[structopt(long, short = "o", parse(from_os_str))]
    /// output dir, like ~/, output filename like ~/manifest_snapshot.csv
    pub output: PathBuf,
    #[structopt(long, short = "i", parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub db_path: PathBuf,
    /// export snapshot block number
    #[structopt(long, short = "b")]
    pub number: Option<BlockNumber>,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "apply-snapshot", about = "apply snapshot")]
pub struct ApplySnapshotOptions {
    #[structopt(long, short = "n")]
    /// Chain Network
    pub net: BuiltinNetworkID,
    #[structopt(long, short = "o", parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/main
    pub to_path: PathBuf,
    #[structopt(long, short = "i", parse(from_os_str))]
    /// manifest file, like manifest-snapshot.csv
    pub manifest_file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let cmd = match opt.cmd {
        Some(cmd) => cmd,
        None => {
            Opt::clap().print_help().ok();
            return Ok(());
        }
    };

    if let Cmd::Exporter(option) = cmd {
        let output = option.output.as_deref();
        let mut writer_builder = csv::WriterBuilder::new();
        let writer_builder = writer_builder.delimiter(b'\t').double_quote(false);
        let result = match output {
            Some(output) => {
                let writer = writer_builder.from_path(output)?;
                export(
                    option.db_path.display().to_string().as_str(),
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

    if let Cmd::Checkkey(option) = cmd {
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

    if let Cmd::ExportBlockRange(option) = cmd {
        let result = export_block_range(
            option.db_path,
            option.output,
            option.net,
            option.start,
            option.end,
        );
        return result;
    }

    if let Cmd::ApplyBlock(option) = cmd {
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

    if let Cmd::StartupInfoBack(option) = cmd {
        let result = startup_info_back(option.to_path, option.back_size, option.net);
        return result;
    }

    if let Cmd::GenBlockTransactions(option) = cmd {
        let result = gen_block_transactions(
            option.to_path,
            option.block_num,
            option.trans_num,
            option.txn_type,
        );
        return result;
    }

    if let Cmd::ExportSnapshot(option) = cmd {
        let result = export_snapshot(option.db_path, option.output, option.net);
        return result;
    }

    if let Cmd::ApplySnapshot(option) = cmd {
        let result = apply_snapshot(option.to_path, option.manifest_file, option.net);
        return result;
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
    let db_stoarge = DBStorage::open_with_cfs(
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
        db_stoarge,
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
            load_bar.set_message(format!("load block {}", num).as_str());
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
        bar.set_message(format!("write block {}", block.header().number()).as_str());
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
    let net = ChainNetwork::new_builtin(network);
    let db_stoarge = DBStorage::new(to_dir.join("starcoindb/db"), RocksdbConfig::default(), None)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_stoarge,
    ))?);
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
        bar.set_message(format!("apply block {}", block_number).as_str());
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
    let db_stoarge = DBStorage::new(to_dir.join("starcoindb/db"), RocksdbConfig::default(), None)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_stoarge,
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
    let back_size = if back_size < BACK_SIZE {
        BACK_SIZE
    } else {
        back_size
    };

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
    ::logger::init();
    let net = ChainNetwork::new_builtin(BuiltinNetworkID::Halley);
    let db_stoarge = DBStorage::new(to_dir.join("starcoindb/db"), RocksdbConfig::default(), None)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_stoarge,
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

/// manifest_file layout
/// block block.header.hash
/// block_accumulator accumulator_root_hash
/// txn_accumulator accumulator_root_hash
/// state           state_root_hash
pub fn export_snapshot(
    from_dir: PathBuf,
    output: PathBuf,
    network: BuiltinNetworkID,
) -> anyhow::Result<()> {
    let net = ChainNetwork::new_builtin(network);
    let db_stoarge = DBStorage::open_with_cfs(
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
        db_stoarge,
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
    let num = chain.status().head().number();
    let cur_num = if num <= SNAP_GAP { num } else { num - SNAP_GAP };
    println!("snapshot block height {}", cur_num);
    let block = chain
        .get_block_by_number(cur_num)?
        .ok_or_else(|| format_err!("get block {} error", cur_num))?;
    let block_info = chain
        .get_block_info_by_number(cur_num)?
        .ok_or_else(|| format_err!("get block info {} error", cur_num))?;
    let state_root = block.header.state_root();
    //let state_tree = StateTree::<AccountAddress>::new(storage.clone(), Some(state_root));
    let statedb = ChainStateDB::new(storage.clone(), Some(state_root));

    let mut mainfest_list = vec![];
    // save block
    let bar = ProgressBar::new(cur_num);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
    );
    let filename = format!("snapshot_block_{}.csv", cur_num);
    mainfest_list.push((output.join(filename.clone()), block.header.id()));
    let mut file = File::create(output.join(filename))?;
    for i in 1..=cur_num {
        let block = chain
            .get_block_by_number(i)?
            .ok_or_else(|| format_err!("get block {} error", i))?;

        writeln!(file, "{}", serde_json::to_string(&block)?)?;
        bar.set_message(format!("save block {}", block.header().number()).as_str());
        bar.inc(1);
    }
    file.flush()?;
    bar.finish();

    // save block_accumulator
    let block_accumulator_info = block_info.get_block_accumulator_info();
    // XXX FIXME
    let block_accumulator_leaves =
        chain
            .get_block_accumulator()
            .get_leaves(1, false, block_accumulator_info.num_leaves - 1)?;
    let filename = format!(
        "snapshot_{}_{}.csv",
        BLOCK_ACCUMULATOR_NODE_PREFIX_NAME, cur_num
    );
    mainfest_list.push((
        output.join(filename.clone()),
        block_accumulator_info.accumulator_root,
    ));
    let bar = ProgressBar::new(block_accumulator_info.num_leaves);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
    );
    let mut file = File::create(output.join(filename))?;
    let mut index = 1;
    for hash in block_accumulator_leaves.iter() {
        writeln!(file, "{}", hash)?;
        bar.set_message(format!("save block accumulator {}", index).as_str());
        bar.inc(1);
        index += 1;
    }
    file.flush()?;
    bar.finish();

    // save transaction_accumulator
    let txn_accumulator_info = block_info.get_txn_accumulator_info();
    let filename = format!(
        "snapshot_{}_{}.csv",
        TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME, cur_num
    );
    // XXX FIXME
    let txn_accumulator_leaves =
        chain
            .get_txn_accumulator()
            .get_leaves(1, false, txn_accumulator_info.num_leaves - 1)?;
    mainfest_list.push((
        output.join(filename.clone()),
        txn_accumulator_info.accumulator_root,
    ));
    let bar = ProgressBar::new(txn_accumulator_info.num_leaves);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
    );
    let mut file = File::create(output.join(filename))?;
    index = 0;
    for hash in txn_accumulator_leaves.iter() {
        writeln!(file, "{}", hash)?;
        bar.set_message(format!("save txn accumulator {}", index).as_str());
        bar.inc(1);
        index += 1;
    }
    file.flush()?;
    bar.finish();

    // contract event (todo)

    // get all transaction (todo)

    // get state
    let filename = format!("snapshot_{}_{}.csv", STATE_NODE_PREFIX_NAME, cur_num);
    mainfest_list.push((
        output.join(filename.clone()),
        state_root,
    ));
    let mut file = File::create(output.join(filename))?;
    let global_states = statedb.dump()?;
    let bar = ProgressBar::new(global_states.len() as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
    );
    index = 1;
    for (account_address, account_state_set) in global_states.into_inner() {
        writeln!(file, "{} {}", serde_json::to_string(&account_address)?, serde_json::to_string(&account_state_set)?)?;
        bar.set_message(format!("write state {}", index).as_str());
        index += 1;
        bar.inc(1);
    }
    file.flush()?;
    bar.finish();

    // save manifest
    let filename = format!("manifest_snapshot.csv");
    let mut file = File::create(output.join(filename))?;

    // XXX FIXME BUG remove ""
    for (path, hash) in mainfest_list.iter() {
        writeln!(file, "{:?} {}", path.to_str().unwrap(), hash)?;
    }
    file.flush()?;

    Ok(())
}

pub fn apply_snapshot(
    to_dir: PathBuf,
    manifest_file: PathBuf,
    network: BuiltinNetworkID,
) -> anyhow::Result<()> {
    let net = ChainNetwork::new_builtin(network);
    let db_stoarge = DBStorage::new(to_dir.join("starcoindb/db"), RocksdbConfig::default(), None)?;
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_stoarge,
    ))?);
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
    if cur_num > 0 {
        println!("cur_num {} expect 0", cur_num);
        std::process::exit(2);
    }
    let mut block_hash = None;
    let reader = BufReader::new(File::open(manifest_file)?);
    for record in reader.lines() {
        let record = record?;
        let strs: Vec<&str> = record.split(" ").collect();
        let file_name = strs[0];
        let hash = strs[1];
        let reader = BufReader::new(File::open(PathBuf::from(file_name))?);
        println!("file_name {} hash {}", file_name, hash);
        let verify_hash = HashValue::from_hex_literal(hash)?;
        if file_name.contains("snapshot_block") {
            for line in reader.lines() {
                let line = line?;
                let block: Block = serde_json::from_str(line.as_str())?;
                block_hash = Some(block.id());
                storage.commit_block(block)?;
            }

            if Some(verify_hash) == block_hash {
                println!("snapshot_block hash match");
            } else {
                println!("snapshot_block hash not match");
                std::process::exit(1);
            }
        } else if file_name.contains(BLOCK_ACCUMULATOR_NODE_PREFIX_NAME) {
            // XXX FIXME
            for line in reader.lines() {
                let line = line?;
                chain
                    .get_block_accumulator()
                    .append(&[HashValue::from_hex_literal(line.as_str())?])?;
            }
            chain.get_block_accumulator().flush()?;
            println!("block accumulator hash {}", chain.get_txn_accumulator().root_hash());
            if chain.get_block_accumulator().root_hash() == verify_hash {
                println!("snapshot_{} hash match", BLOCK_ACCUMULATOR_NODE_PREFIX_NAME);
            } else {
                println!("snapshot_{} hash not match", BLOCK_ACCUMULATOR_NODE_PREFIX_NAME);
                std::process::exit(1);
            }
        } else if file_name.contains(TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME) {
            // XXX FIXME
            for line in reader.lines() {
                let line = line?;
                chain
                    .get_txn_accumulator()
                    .append(&[HashValue::from_hex_literal(line.as_str())?])?;
            }
            chain.get_txn_accumulator().flush()?;
            if chain.get_txn_accumulator().root_hash() == verify_hash {
                println!("snapshot_{} hash match", TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME);
            } else {
                println!("snapshot_{} hash not match", TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME);
                std::process::exit(1);
            }
        } else {
            let mut account_states = vec![];
            for line in reader.lines() {
                let line = line?;
                let strs: Vec<&str> = line.split(" ").collect();
                let account_address : AccountAddress = serde_json::from_str(strs[0])?;
                let account_state_set : AccountStateSet = serde_json::from_str(strs[1])?;
                account_states.push((account_address, account_state_set));
            }
            let chain_state_set = ChainStateSet::new(account_states);
            chain.chain_state().apply(chain_state_set)?;
            if chain.chain_state_reader().state_root() == verify_hash {
                println!("snapshot_state hash match");
            } else {
                println!("snapshot_state hash match");
                std::process::exit(1);
            }
        }
    }

    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("load blocks from file use time: {:?}", use_time.as_millis());
    let start_time = SystemTime::now();
    let bar = ProgressBar::new(5);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
    );
    bar.finish();
    if let Some(block_hash) = block_hash {
        let startup_info = StartupInfo::new(block_hash);
        storage.save_startup_info(startup_info)?;
    }
    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("apply block use time: {:?}", use_time.as_secs());
    Ok(())
}
