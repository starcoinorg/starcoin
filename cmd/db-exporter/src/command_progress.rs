use anyhow::{bail, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_config::ChainNetwork;
use starcoin_crypto::HashValue;
use starcoin_genesis::Genesis;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{Storage, StorageVersion};
use starcoin_types::block::{Block, BlockNumber};
use starcoin_vm_types::language_storage::TypeTag;
use starcoin_vm_types::vm_status::KeptVMStatus;
use std::io::{Seek, SeekFrom};
use std::sync::Arc;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    time::SystemTime,
};

struct CommandResult {
    succeed: usize,
    failed: usize,
}

impl CommandResult {
    pub fn new(succeed: usize, failed: usize) -> CommandResult {
        CommandResult { succeed, failed }
    }
}

pub struct ParallelCommandFilter {
    signer: Option<String>,
    func_name: Option<String>,
    // function name for filter, none for all
    ty_args: Option<Vec<String>>,
    // template parameter for filter, none for all
    args: Option<Vec<String>>, // arguments type for filter, none for all
}

impl ParallelCommandFilter {
    pub fn new(
        signer: &Option<String>,
        func_name: &Option<String>,
        ty_args: &Option<Vec<String>>,
        args: &Option<Vec<String>>,
    ) -> Option<Self> {
        if signer.is_some() || func_name.is_some() || ty_args.is_some() || args.is_some() {
            Some(ParallelCommandFilter {
                signer: signer.clone(),
                func_name: func_name.clone(),
                ty_args: ty_args.clone(),
                args: args.clone(),
            })
        } else {
            None
        }
    }

    pub fn match_signer(&self, signer: &str) -> bool {
        if self.signer.is_none() {
            return true;
        }
        self.signer.clone().unwrap() == signer
    }

    pub fn match_func_name(&self, func_name: &str) -> bool {
        if self.func_name.is_none() {
            return true;
        }
        self.func_name.clone().unwrap() == func_name
    }

    pub fn match_ty_args(&self, _ty_args: &[TypeTag]) -> bool {
        if self.ty_args.is_some() {
            print!("match_ty_args |  {:?}", self.ty_args);
            // TODO(Bob): To Compare
        }
        true
    }

    pub fn match_args(&self, _args: &[Vec<u8>]) -> bool {
        if self.args.is_some() {
            print!("match_args |  {:?}", self.args);
            // TODO(Bob): To Compare
        }
        true
    }
}

pub struct ParallelCommandReadBodyFromExportLine {
    file: File,
    line_count: u64,
}

impl ParallelCommandReadBodyFromExportLine {
    fn count_lines(reader: &mut BufReader<File>) -> Result<u64> {
        let line_count = reader.lines().count();
        reader.seek(SeekFrom::Start(0))?;
        Ok(line_count as u64)
    }

    pub fn new(input_path: PathBuf) -> Result<Self> {
        let file = File::open(input_path.display().to_string())?;
        let line_count = ParallelCommandReadBodyFromExportLine::count_lines(&mut BufReader::new(
            file.try_clone()?,
        ))?;
        Ok(Self { file, line_count })
    }
}

impl ParallelCommandBlockReader for ParallelCommandReadBodyFromExportLine {
    fn get_progress_interval(&self) -> u64 {
        self.line_count
    }

    fn read(&self, load_bar: &ProgressBar) -> Result<Vec<Block>> {
        let reader = BufReader::new(self.file.try_clone()?);
        let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;
        Ok(lines
            .par_iter()
            .map(|line| {
                let ret = serde_json::from_str::<Block>(line.as_str());
                load_bar.inc(1);
                Ok(ret)?
            })
            .collect::<Result<Vec<Block>, _>>()?)
    }

    fn query_txn_exec_state(&self, _txn_hash: HashValue) -> String {
        "OK".to_string()
    }
}

fn start_end_check(start: u64, end: u64, cur_num: u64) -> (u64, u64) {
    if start == 0 && end == 0 {
        (0, cur_num)
    } else {
        assert!(start < end, "End must bigger than Start");
        assert!(end <= cur_num, "End number should less than Head num");
        (start, end)
    }
}

#[test]
fn test_start_end_check() {
    let (start, end) = start_end_check(0, 100, 100);
    assert_eq!(start, 0);
    assert_eq!(end, 100);

    let (start, end) = start_end_check(0, 0, 100);
    assert_eq!(start, 0);
    assert_eq!(end, 100);
}

#[test]
#[should_panic = "End must bigger than Start"]
fn test_start_end_check_panic_invalid() {
    start_end_check(100, 1, 100);
}

#[test]
#[should_panic = "End must bigger than Start"]
fn test_start_end_check_panic_invalid2() {
    start_end_check(100, 100, 100);
}

#[test]
#[should_panic = "End number should less than Head num"]
fn test_start_end_check_panic_overflow() {
    start_end_check(100, 500, 100);
}

pub struct ParallelCommandReadBlockFromDB {
    start_num: u64,
    end_num: u64,
    chain: Arc<BlockChain>,
    skip_empty_block: bool,
}

impl ParallelCommandReadBlockFromDB {
    pub fn new(
        input_path: PathBuf,
        net: ChainNetwork,
        start: u64,
        end: u64,
        skip_empty_block: bool,
    ) -> Result<(Self, Arc<Storage>)> {
        let storage = Self::init_db_obj(input_path.clone()).expect("Failed to initialize db");
        let (chain_info, _) =
            Genesis::init_and_check_storage(&net, storage.clone(), input_path.as_ref())
                .expect("Failed init_and_check_storage");
        let chain = BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage.clone(),
            None,
        )
        .expect("Failed to initialize block chain");

        let (start_num, end_num) = start_end_check(start, end, chain.status().head().number());
        Ok((
            Self {
                start_num,
                end_num,
                chain: Arc::new(chain),
                skip_empty_block,
            },
            storage,
        ))
    }
    fn init_db_obj(db_path: PathBuf) -> Result<Arc<Storage>> {
        let db_storage = DBStorage::open_with_cfs(
            db_path.join("starcoindb/db/starcoindb"),
            StorageVersion::current_version()
                .get_column_family_names()
                .to_vec(),
            true,
            Default::default(),
            None,
        )?;
        Ok(Arc::new(Storage::new(
            StorageInstance::new_cache_and_db_instance(CacheStorage::new(None), db_storage),
        )?))
    }
}

impl ParallelCommandBlockReader for ParallelCommandReadBlockFromDB {
    fn get_progress_interval(&self) -> u64 {
        self.end_num - self.start_num
    }

    fn read(&self, load_bar: &ProgressBar) -> Result<Vec<Block>> {
        println!(
            "ParallelCommandBlockReader::read | read range: {}, {}, skip empty block: {}",
            self.start_num, self.end_num, self.skip_empty_block
        );

        Ok((self.start_num..=self.end_num)
            .collect::<Vec<BlockNumber>>()
            .par_iter()
            .filter_map(|num| {
                load_bar.inc(1);
                if self.skip_empty_block {
                    self.chain
                        .get_block_by_number(*num)
                        .ok()
                        .flatten()
                        .filter(|block| !block.transactions().is_empty())
                } else {
                    self.chain.get_block_by_number(*num).ok().flatten()
                }
            })
            .collect::<Vec<Block>>())
    }

    fn query_txn_exec_state(&self, txn_hash: HashValue) -> String {
        let txn = self
            .chain
            .get_transaction_info(txn_hash)
            .expect("Query failed!");
        match txn {
            Some(info) => match info.status {
                KeptVMStatus::Executed => "OK".to_string(),
                _ => "FALIED".to_string(),
            },
            None => "CANT_FOUND_TXN".to_string(),
        }
    }
}

pub struct ParallelCommandProgress {
    name: String,
    parallel_level: usize,
    block_reader: Arc<dyn ParallelCommandBlockReader>,
    filter: Option<ParallelCommandFilter>,
    obs: Option<Arc<dyn ParallelCommandObserver>>,
}

impl ParallelCommandProgress {
    pub fn new(
        name: String,
        parallel_level: usize,
        reader: Arc<dyn ParallelCommandBlockReader>,
        filter: Option<ParallelCommandFilter>,
        obs: Option<Arc<dyn ParallelCommandObserver>>,
    ) -> ParallelCommandProgress {
        Self {
            name,
            block_reader: reader.clone(),
            parallel_level,
            filter,
            obs,
        }
    }

    pub fn progress<CommandT: Sync + Send, ErrorT>(self, command: &CommandT) -> Result<()>
    where
        Block: ParallelCommand<CommandT, ErrorT>,
    {
        println!("Start progress task, batch_size: {:?}", self.parallel_level);

        let mut start_time = SystemTime::now();
        //let file_name = self.file_path.display().to_string();
        //let reader = BufReader::new(File::open(file_name)?);
        println!(
            "Reading file process expire mini seconds time: {:?}",
            SystemTime::now().duration_since(start_time)?.as_micros()
        );

        start_time = SystemTime::now();

        if let Some(observer) = &self.obs {
            observer.before_progress()?;
        }

        let load_interval_count = self.block_reader.get_progress_interval();
        let load_bar = ProgressBar::new(load_interval_count).with_style(
            ProgressStyle::default_bar()
                .template("loading [{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
        );

        let mut all_items = self.block_reader.read(&load_bar)?;
        load_bar.finish();

        let all_item_size = all_items.len();
        all_items.retain(|b| (*b).matched(&self.filter));
        let filtered_item_size = all_items.len();

        println!(
            "Reading lines from file expire time: {:?}, get interval: {:?},  actual return item size: {:?}, filtered item size:{:?}",
            SystemTime::now().duration_since(start_time)?.as_secs(),
            load_interval_count,
            all_item_size,
            filtered_item_size,
        );

        // It is necessary to divide all rows into subsets
        // when reading them,
        // so that they can be divided into several threads for the following operations
        start_time = SystemTime::now();

        let progress_bar = ProgressBar::new(all_items.len() as u64).with_style(
            ProgressStyle::default_bar()
                .template("processing [{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
        );
        let excution_result = all_items
            .into_par_iter()
            .chunks(self.parallel_level)
            .map(|item_vec| {
                item_vec
                    .into_iter()
                    .map(|item| {
                        let (succeed, failed) = item.execute(self.block_reader.as_ref(), command);
                        progress_bar.inc(1);
                        CommandResult::new(succeed, failed.len())
                    })
                    .collect::<Vec<CommandResult>>()
            })
            .collect::<Vec<Vec<CommandResult>>>();

        let result = excution_result.into_iter().flatten().fold(
            CommandResult {
                succeed: 0,
                failed: 0,
            },
            |acc, result| CommandResult {
                succeed: acc.succeed + result.succeed,
                failed: acc.failed + result.failed,
            },
        );
        progress_bar.finish();

        println!("Running ParallelCommand {:?},  use time: {:?}, success modules: {}, error modules: {}, total modules: {}",
                 self.name, SystemTime::now().duration_since(start_time)?.as_secs(), result.succeed, result.failed, result.succeed + result.failed);
        if result.failed > 0 {
            bail!("verify block modules error");
        }
        if let Some(observer) = &self.obs {
            observer.after_progress()?;
        }

        Ok(())
    }
}

pub trait ParallelCommand<CommandT, ErrorT> {
    fn execute(
        &self,
        block_reader: &dyn ParallelCommandBlockReader,
        cmd: &CommandT,
    ) -> (usize, Vec<ErrorT>);

    fn matched(&self, filter: &Option<ParallelCommandFilter>) -> bool;
}

pub trait ParallelCommandObserver {
    fn before_progress(&self) -> Result<()>;
    fn after_progress(&self) -> Result<()>;
}

pub trait ParallelCommandBlockReader: Sync + Send {
    fn get_progress_interval(&self) -> u64;
    fn read(&self, load_bar: &ProgressBar) -> Result<Vec<Block>>;
    fn query_txn_exec_state(&self, txn_hash: HashValue) -> String;
}
