use anyhow::bail;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::SystemTime;

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
    pub fn new(name: String, file_path: PathBuf, batch_size: usize) -> CmdBatchExecution {
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
        let reader = BufReader::new(File::open(file_name)?);
        println!(
            "Reading file process expire mini seconds time: {:?}",
            SystemTime::now().duration_since(start_time)?.as_micros()
        );

        start_time = SystemTime::now();
        let lines = reader.lines().collect::<Result<Vec<_>, _>>()?;

        let all_items = lines
            .par_iter()
            .map(|line| Ok(serde_json::from_str::<BodyT>(line.as_str()))?)
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
            .chunks(self.batch_size)
            .map(|item_vec| {
                item_vec
                    .into_iter()
                    .map(|item| {
                        let (succeed, failed) = item.execute();
                        progress_bar.inc(1);
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
