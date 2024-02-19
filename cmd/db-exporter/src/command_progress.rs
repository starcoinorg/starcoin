use anyhow::{bail, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use starcoin_vm_types::language_storage::TypeTag;
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

pub trait ParallelCommandObserver {
    fn before_progress(&self) -> Result<()>;
    fn after_progress(&self) -> Result<()>;
}

pub struct ParallelCommandProgress {
    name: String,
    file_path: PathBuf,
    parallel_level: usize,
    filter: Option<ParallelCommandFilter>,
    obs: Option<Arc<dyn ParallelCommandObserver>>,
}

impl ParallelCommandProgress {
    pub fn new(
        name: String,
        file_path: PathBuf,
        parallel_level: usize,
        filter: Option<ParallelCommandFilter>,
        obs: Option<Arc<dyn ParallelCommandObserver>>,
    ) -> ParallelCommandProgress {
        Self {
            file_path,
            name,
            parallel_level,
            filter,
            obs,
        }
    }

    pub fn progress<CommandT, BodyT, ErrorT>(self, command: &CommandT) -> Result<()>
    where
        BodyT: ParallelCommand<CommandT, BodyT, ErrorT>
            + Send
            + Sync
            + Clone
            + serde::Serialize
            + for<'a> serde::Deserialize<'a>
            + 'static,
    {
        println!("Start progress task, batch_size: {:?}", self.parallel_level);

        let mut start_time = SystemTime::now();
        let file_name = self.file_path.display().to_string();
        let reader = BufReader::new(File::open(file_name)?);
        println!(
            "Reading file process expire mini seconds time: {:?}",
            SystemTime::now().duration_since(start_time)?.as_micros()
        );

        start_time = SystemTime::now();
        let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;

        if let Some(observer) = &self.obs {
            observer.before_progress()?;
        }

        let all_items = lines
            .par_iter()
            .map(|line| Ok(serde_json::from_str::<BodyT>(line.as_str()))?)
            .filter(|item| item.unwrap().matched(self.filters))
            .collect::<Result<Vec<BodyT>, _>>()?;

        let progress_bar = ProgressBar::new(all_items.len() as u64).with_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
        );

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

pub trait ParallelCommand<CommandT, BodyT, ErrorT> {
    fn execute(&self, cmd: &CommandT) -> (usize, Vec<ErrorT>);

    fn before_command(&self, cmd: &CommandT) -> Result<()>;

    fn after_command(&self, cmd: &CommandT) -> Result<()>;

    fn matched(&self, filter: Option<ParallelCommandFilter>) -> bool;
}
