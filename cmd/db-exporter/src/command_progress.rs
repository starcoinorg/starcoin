use anyhow::{bail, format_err, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_config::ChainNetwork;
use starcoin_genesis::Genesis;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{Storage, StorageVersion};
use starcoin_types::block::{Block, BlockNumber};
use starcoin_vm_types::language_storage::TypeTag;
use std::sync::Arc;
use std::{fs::File, io::{BufRead, BufReader}, path::PathBuf, time::SystemTime};
use std::io::{Seek, SeekFrom};

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
    fn new(
        signer: Option<String>,
        func_name: Option<String>,
        ty_args: Option<Vec<String>>,
        args: Option<Vec<String>>,
    ) -> Option<Self> {
        if func_name.is_some() || ty_args.is_some() || args.is_some() {
            Some(ParallelCommandFilter {
                signer,
                func_name,
                ty_args,
                args,
            })
        } else {
            None
        }
    }

    pub fn match_signer(&self, signer: &str) -> bool {
        self.signer.as_ref().map_or(false, |n| n == signer)
    }

    pub fn match_func_name(&self, func_name: &str) -> bool {
        self.func_name.as_ref().map_or(false, |n| n == func_name)
    }

    pub fn match_ty_args(&self, _ty_args: &Vec<TypeTag>) -> bool {
        // TODO(Bob): To Compare
        true
    }

    pub fn match_args(&self, _args: &Vec<Vec<u8>>) -> bool {
        // TODO(Bob): To Compare
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
        let line_count = ParallelCommandReadBodyFromExportLine::count_lines(&mut BufReader::new(file.try_clone()?))?;
        Ok(Self {
            file,
            line_count,
        })
    }
}

impl ParallelCommandBlockReader for ParallelCommandReadBodyFromExportLine {
    fn get_progress_interval(&self) -> u64 {
        self.line_count
    }

    fn read(&self) -> Result<Vec<Block>> {
        let reader = BufReader::new(self.file.try_clone()?);
        let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;
        Ok(lines
            .par_iter()
            .map(|line| Ok(serde_json::from_str::<Block>(line.as_str()))?)
            .collect::<Result<Vec<Block>, _>>()?)
    }
}

pub struct ParallelCommandReadBlockFromDB {
    start_num: u64,
    end_num: u64,
    chain: Arc<BlockChain>,
}

const BLOCK_GAP: u64 = 1000;

impl ParallelCommandReadBlockFromDB {
    pub fn new(
        input_path: PathBuf,
        net: ChainNetwork,
        start: u64,
        end: u64,
    ) -> Result<(Self, Arc<Storage>)> {
        let storage = Self::init_db_obj(input_path.clone()).expect("Failed to initialize db");
        let (chain_info, _) =
            Genesis::init_and_check_storage(&net, storage.clone(), input_path.as_ref())
                .expect("Failed init_and_check_storage");
        let chain = BlockChain::new(net.time_service(), chain_info.head().id(), storage.clone(), None)
            .expect("Failed to initialize block chain");

        let cur_num = chain.status().head().number();

        let (start_num, end_num) = if start != 0 && end == 0 {
            (0, cur_num)
        } else {
            let end = if cur_num > end + BLOCK_GAP {
                end
            } else if cur_num > BLOCK_GAP {
                cur_num - BLOCK_GAP
            } else {
                end
            };
            (start, end)
        };

        if start > cur_num || start > end {
            return Err(format_err!(
                "cur_num {} start {} end {} illegal",
                cur_num,
                start,
                end
            ));
        };

        Ok((
            Self {
                start_num,
                end_num,
                chain: Arc::new(chain),
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

    fn read(&self) -> Result<Vec<Block>> {
        let ret = (self.start_num..=self.end_num)
            .collect::<Vec<BlockNumber>>()
            .into_iter()
            .map(|num| {
                // progress_bar.set_message(format!("load block {}", num));
                // progress_bar.inc(1);
                self.chain.get_block_by_number(num).ok()?
            })
            .filter(|block| block.is_some())
            .map(|block| block.unwrap())
            .collect();
        Ok(ret)
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
        // let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;

        // let all_items = lines
        //     .par_iter()
        //     .map(|line| Ok(serde_json::from_str::<BodyT>(line.as_str()))?)
        //     .filter(|item| match item {
        //         Ok(i) => i.matched(&self.filter),
        //         Err(_e) => false,
        //     })
        //     .collect::<Result<Vec<BodyT>, _>>()?;
        if let Some(observer) = &self.obs {
            observer.before_progress()?;
        }

        let progress_bar = ProgressBar::new(self.block_reader.get_progress_interval()).with_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
        );

        let all_items = self.block_reader.read()?;
        // .iter()
        // .filter(|b| (*b).matched(&self.filter))
        // .map(|b| *b)
        // .collect();

        println!(
            "Reading lines from file expire time: {:?}",
            SystemTime::now().duration_since(start_time)?.as_secs()
        );

        // It is necessary to divide all rows into subsets
        // when reading them,
        // so that they can be divided into several threads for the following operations
        start_time = SystemTime::now();

        let excution_result = all_items
            .into_par_iter()
            .chunks(self.parallel_level)
            .map(|item_vec| {
                item_vec
                    .into_iter()
                    .map(|item| {
                        let (succeed, failed) = item.execute(command);
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

        println!("verify {:?},  use time: {:?}, success modules: {}, error modules: {}, total modules: {}",
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
    fn execute(&self, cmd: &CommandT) -> (usize, Vec<ErrorT>);

    fn matched(&self, filter: &Option<ParallelCommandFilter>) -> bool;
}

pub trait ParallelCommandObserver {
    fn before_progress(&self) -> Result<()>;
    fn after_progress(&self) -> Result<()>;
}

pub trait ParallelCommandBlockReader {
    fn get_progress_interval(&self) -> u64;
    fn read(&self) -> Result<Vec<Block>>;
}
