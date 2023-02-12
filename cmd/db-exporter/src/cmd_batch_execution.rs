use anyhow::bail;
use atomic_counter::AtomicCounter;
use futures::executor::block_on;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::{join, task};
use std::collections::VecDeque;

pub struct CmdBatchExecution {
    name: String,
    file_path: PathBuf,
    task_count: usize,
    progress_bar: Option<ProgressBar>,
}

const DEFAULT_CSV_SIZE: usize = 20000;

impl CmdBatchExecution {
    pub fn new(
        name: String,
        file_path: PathBuf,
        show_progress_bar: bool,
        task_count: usize,
    ) -> CmdBatchExecution {
        let progress_bar = if show_progress_bar {
            Option::Some(ProgressBar::new(0))
        } else {
            None
        };
        Self {
            file_path,
            progress_bar,
            name,
            task_count,
        }
    }

    pub fn progress<CmdT, BodyT, ErrorT>(self) -> anyhow::Result<()>
        where
            BodyT: BatchCmdExec<CmdT, BodyT, ErrorT>
            + Send
            + Clone
            + serde::Serialize
            + for<'a> serde::Deserialize<'a>
            + 'static
    {
        //let start_time = SystemTime::now();
        let file_name = self.file_path.display().to_string();
        let mut reader = BufReader::new(File::open(file_name.clone())?);

        let mut all_items = VecDeque::with_capacity(DEFAULT_CSV_SIZE);
        for line in reader.lines() {
            let line = line?;
            let item: BodyT = serde_json::from_str::<BodyT>(line.as_str())?;
            all_items.push_back(item);
        }

        // It is necessary to divide all rows into subsets
        // when reading them,
        // so that they can be divided into several threads for the following operations
        let mut i = 0;
        let line_cnt = all_items.len();
        let subset_capcity = line_cnt / self.task_count;
        let mut all_subsets = VecDeque::with_capacity(self.task_count);
        all_subsets.push_back(vec![]);
        while !all_items.is_empty() {
            let subset_count = all_subsets.len();
            if (i / subset_capcity) > subset_count {
                all_subsets.push_back(vec![]);
            }
            let front_item = all_items.pop_front();
            if front_item.is_some() {
                all_subsets.back_mut().unwrap().push(front_item.unwrap());
            }
            i = i + 1;
        }

        let start_time = SystemTime::now();
        // let start_time =
            // if self.progress_bar.is_some() {
            //     self.progress_bar.as_mut().unwrap().start(line_cnt)
            // } else {
            //     SystemTime::now()
            // };

        let success_counter = Arc::new(atomic_counter::RelaxedCounter::new(0));
        let error_counter = Arc::new(atomic_counter::RelaxedCounter::new(0));
        let mut join_handlers = vec![];

        while !all_subsets.is_empty() {
            let subset = all_subsets.pop_front();
            //let total_modules = success_counter.get() + error_counter.get();
            let success_counter = success_counter.clone();
            let error_counter = error_counter.clone();
            join_handlers.push(task::spawn(async move {
                for item in subset.unwrap() {
                    let (success_count, errors) = item.execute();
                    success_counter.add(success_count);
                    error_counter.add(errors.len());

                    // if self.progress_bar.is_some() {
                    //     self.progress_bar.as_mut().unwrap().advance(1);
                    // }
                }
            }));
        }

        // Wait all feature finished
        for handler in join_handlers {
            block_on(handler)?;
        }

        // if self.progress_bar.is_some() {
        //     self.progress_bar.as_mut().unwrap().end();
        // };

        let use_time = SystemTime::now().duration_since(start_time)?;
        println!("verify {:?},  use time: {:?}, success modules: {}, error modules: {}, total modules: {}",
                 self.name, use_time.as_secs(), success_counter.get(), error_counter.get(), success_counter.get() + error_counter.get());
        if error_counter.get() > 0 {
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

impl ProgressBarExtension for ProgressBar  {
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