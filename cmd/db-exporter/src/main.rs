// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use bcs_ext::Sample;
use csv::Writer;
use starcoin_chain::verifier::{
    BasicVerifier, ConsensusVerifier, FullVerifier, NoneVerifier, Verifier,
};
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_config::{BuiltinNetworkID, ChainNetwork, RocksdbConfig};
use starcoin_crypto::HashValue;
use starcoin_genesis::Genesis;
use starcoin_storage::block::FailedBlock;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::ValueCodec;
use starcoin_storage::storage::{InnerStore, StorageInstance};
use starcoin_storage::{
    BlockStore, Storage, BLOCK_HEADER_PREFIX_NAME, BLOCK_PREFIX_NAME, FAILED_BLOCK_PREFIX_NAME,
    VEC_PREFIX_NAME,
};
use starcoin_types::block::{Block, BlockHeader, BlockNumber};
use starcoin_types::startup_info::StartupInfo;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;
use structopt::StructOpt;

pub fn export<W: std::io::Write>(
    db: &str,
    mut csv_writer: Writer<W>,
    schema: DbSchema,
) -> anyhow::Result<()> {
    let db_storage =
        DBStorage::open_with_cfs(db, VEC_PREFIX_NAME.to_vec(), true, Default::default(), None)?;
    let mut iter = db_storage.iter(schema.to_string().as_str())?;
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
    /// output file, like block.csv
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
    /// input file, like accounts.csv
    pub input_path: PathBuf,
    #[structopt(possible_values = &Verifier::variants(), case_insensitive = true)]
    /// Verify type:  Basic, Consensus, Full, None, eg.
    pub verifier: Option<Verifier>,
    #[structopt(long, short = "w")]
    /// Watch metrics logs.
    pub watch: bool,
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
            VEC_PREFIX_NAME.to_vec(),
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
        match guard.report().build() {
            Ok(report) => {
                let file = File::create("/tmp/flamegraph.svg").unwrap();
                report.flamegraph(file).unwrap();
            }
            Err(_) => {}
        }
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
        VEC_PREFIX_NAME.to_vec(),
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
    let start_time = SystemTime::now();
    let block_list: Result<Vec<Block>> = (start..end)
        .collect::<Vec<BlockNumber>>()
        .into_iter()
        .map(|num| {
            chain
                .get_block_by_number(num)?
                .ok_or_else(|| format_err!("{} get block error", num))
        })
        .collect();
    let block_list = block_list?;
    let mut file = File::create(output)?;
    for block in block_list {
        writeln!(file, "{}", serde_json::to_string(&block)?)?;
    }
    file.flush()?;
    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("export range block use time: {:?}", use_time.as_nanos());
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
    let reader = BufReader::new(File::open(input_path)?);
    let mut blocks = vec![];
    for record in reader.lines() {
        let record = record?;
        let block: Block = serde_json::from_str(record.as_str())?;
        blocks.push(block);
    }
    let mut last_block_hash = None;
    if let Some(last_block) = blocks.last() {
        let start = blocks.get(0).unwrap().header().number();
        let end = last_block.header().number();
        let cur_num = chain.status().head().number();
        println!(
            "current number {}, import [{},{}] block number",
            cur_num, start, end
        );
        last_block_hash = Some(last_block.header.id());
    }
    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("load blocks from file use time: {:?}", use_time.as_millis());
    let start_time = SystemTime::now();
    for block in blocks {
        match verifier {
            Verifier::Basic => chain.apply_with_verifier::<BasicVerifier>(block)?,
            Verifier::Consensus => chain.apply_with_verifier::<ConsensusVerifier>(block)?,
            Verifier::Full => chain.apply_with_verifier::<FullVerifier>(block)?,
            Verifier::None => chain.apply_with_verifier::<NoneVerifier>(block)?,
        };
    }
    if let Some(last_block_hash) = last_block_hash {
        let startup_info = StartupInfo::new(last_block_hash);
        storage.save_startup_info(startup_info)?;
    }
    let use_time = SystemTime::now().duration_since(start_time)?;
    println!("apply block use time: {:?}", use_time.as_secs());
    Ok(())
}
