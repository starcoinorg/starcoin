use anyhow::bail;
use atomic_counter::AtomicCounter;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::task;

struct CmdBatchExecution {
    name: String,
    file_path: PathBuf,
    show_progress_bar: bool,
}
//
// impl CmdBatchExecution {
//     pub fn new(name: String, file_path: PathBuf, show_progress_bar: bool) -> CmdBatchExecution {
//         Self {
//             file_path,
//             show_progress_bar,
//             name,
//         }
//     }
//
//     pub fn progress<
//         CmdT,
//         BodyT: BatchCmdExec<BodyT, CmdT>
//             + Send
//             + Clone
//             + serde::Serialize
//             + for<'a> serde::Deserialize<'a>
//             + 'static,
//         ErrorT,
//     >(
//         &self,
//     ) -> anyhow::Result<()> {
//         let start_time = SystemTime::now();
//         let file_name = self.file_path.display().to_string();
//         let reader = BufReader::new(File::open(file_name.clone())?);
//         let mut items = vec![];
//         for record in reader.lines() {
//             let record = record?;
//             let item: BodyT = serde_json::from_str::<BodyT>(record.as_str())?;
//             items.push(item);
//         }
//         if items.is_empty() {
//             println!("file {} has apply, but empty", file_name);
//             return Ok(());
//         }
//
//         println!("Start progress count: {}", items.len());
//
//         let use_time = SystemTime::now().duration_since(start_time)?;
//         println!("load blocks from file use time: {:?}", use_time.as_millis());
//
//         let start_time = SystemTime::now();
//
//         let mut bar = if self.show_progress_bar {
//             let bar = ProgressBar::new(items.len() as u64);
//             bar.set_style(
//                 ProgressStyle::default_bar()
//                     .template("[{elapsed_precise}] {bar:100.cyan/blue} {percent}% {msg}"),
//             );
//             Some(bar)
//         } else {
//             None
//         };
//
//         let success_counter = Arc::new(atomic_counter::RelaxedCounter::new(0));
//         let error_counter = Arc::new(atomic_counter::RelaxedCounter::new(0));
//         for item in items {
//             //let total_modules = success_counter.get() + error_counter.get();
//             let success_counter = success_counter.clone();
//             let error_counter = error_counter.clone();
//
//             task::spawn(async move {
//                 let (success_count, errors) = item.execute();
//                 // if !errors.is_empty() {
//                 //     println!(
//                 //         "verify item modules {} error modules: {:?}",
//                 //         block_number, errors
//                 //     );
//                 // }
//                 success_counter.add(success_count);
//                 error_counter.add(errors.len());
//             });
//
//             if bar.is_some() {
//                 // bar.set_message(format!(
//                 //     "verify block {} , total_modules: {}",
//                 //     block_number, total_modules
//                 // ));
//                 bar.as_mut().unwrap().inc(1);
//             };
//         }
//
//         if bar.is_some() {
//             let _ = bar.unwrap().finish();
//         };
//
//         let use_time = SystemTime::now().duration_since(start_time)?;
//         println!("verify {:?},  use time: {:?}, success modules: {}, error modules: {}, total modules: {}",
//                  self.name, use_time.as_secs(), success_counter.get(), error_counter.get(), success_counter.get() + error_counter.get());
//         if error_counter.get() > 0 {
//             bail!("verify block modules error");
//         }
//         Ok(())
//     }
// }
//
// pub trait BatchCmdExec<CmdT, BodyT, ErrorT> {
//     fn execute(&self) -> (usize, Vec<ErrorT>);
// }
