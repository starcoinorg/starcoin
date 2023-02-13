use anyhow::bail;
use atomic_counter::AtomicCounter;
use futures::executor::block_on;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::{join, task};

pub struct CmdBatchExecution {
    name: String,
    file_path: PathBuf,
    batch_size: usize,
}

struct ExecutionResult {
    succeed: usize,
    failed: usize,
}

impl ExecutionResult {
    pub fn new(succeed: usize, failed: usize) -> ExecutionResult {
        ExecutionResult { succeed, failed }
    }
}

impl CmdBatchExecution {
    pub fn new(
        name: String,
        file_path: PathBuf,
        batch_size: usize,
    ) -> CmdBatchExecution {
        Self {
            file_path,
            name,
            batch_size,
        }
    }

    pub fn progress<CmdT, BodyT, ErrorT>(self) -> anyhow::Result<()>
    where
        BodyT: BatchCmdExec<CmdT, BodyT, ErrorT>
            + Send
            + Sync
            + Clone
            + serde::Serialize
            + for<'a> serde::Deserialize<'a>
            + 'static,
    {
        println!("Start progress task, batch_size: {:?}", self.batch_size);

        let mut start_time = SystemTime::now();
        let file_name = self.file_path.display().to_string();
        let mut reader = BufReader::new(File::open(file_name.clone())?);
        println!(
            "Reading file process expire mini seconds time: {:?}",
            SystemTime::now().duration_since(start_time)?.as_micros()
        );

        start_time = SystemTime::now();
        let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;

        let mut all_items = lines
            .par_iter()
            .map(|line| Ok(serde_json::from_str::<BodyT>(line.as_str()))?)
            .collect::<Result<Vec<BodyT>, _>>()?;

        let progress_bar = ProgressBar::new(all_items.len() as u64).with_style(ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"));

        println!(
            "Reading lines from file expire time: {:?}",
            SystemTime::now().duration_since(start_time)?.as_secs()
        );

        // It is necessary to divide all rows into subsets
        // when reading them,
        // so that they can be divided into several threads for the following operations
        start_time = SystemTime::now();

        let counter = Arc::new(atomic_counter::RelaxedCounter::new(0));
        let excution_result = all_items
            .into_par_iter()
            .chunks(self.batch_size)
            .map(|item_vec| {
                item_vec
                    .into_iter()
                    .map(|item| {
                        let (succeed, failed) = item.execute();
                        progress_bar.inc(1);
                        //println!("Total processed items: {}", counter.get());
                        ExecutionResult::new(succeed, failed.len())
                    })
                    .collect::<Vec<ExecutionResult>>()
            })
            .collect::<Vec<Vec<ExecutionResult>>>();

        let result = excution_result.into_iter().flatten().fold(
            ExecutionResult {
                succeed: 0,
                failed: 0,
            },
            |acc, result| ExecutionResult {
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
        Ok(())
    }
}

pub trait BatchCmdExec<CmdT, BodyT, ErrorT> {
    fn execute(&self) -> (usize, Vec<ErrorT>);
}

/// Progress bar extension
///
trait ProgressBarExtension {
    fn start(self, capcity: usize) -> SystemTime;
    fn advance(self, cnt: usize);
    fn end(&self);
}

impl ProgressBarExtension for ProgressBar {
    fn start(self, capcity: usize) -> SystemTime {
        println!("Start progress count: {}", capcity);
        //let use_time = SystemTime::now().duration_since(start_time)?;
        //println!("load blocks from file use time: {:?}", use_time.as_millis());

        self.set_length(capcity as u64);
        self.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
        );
        SystemTime::now()
    }

    fn advance(self, cnt: usize) {
        // self.set_message(format!(
        //     "verify item: {} , total_modules: {}",
        //     block_number, total_modules
        // ));
        self.inc(cnt as u64);
    }

    fn end(&self) {
        self.finish();
    }
}
