// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use lazy_static::lazy_static;
use log::LevelFilter;
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        rolling_file::policy::compound::{
            roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
        },
        rolling_file::RollingFileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    Handle,
};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::{Arc, Once};

/// Logger prelude which includes all logging macros.
pub mod prelude {
    pub use log::{debug, error, info, log_enabled, trace, warn, Level, LevelFilter};
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]
struct LoggerConfigArg {
    enable_stderr: bool,
    level: LevelFilter,
    log_path: Option<PathBuf>,
    max_file_size: u64,
    max_backup: u32,
}

impl LoggerConfigArg {
    pub fn new(enable_stderr: bool, level: LevelFilter) -> Self {
        Self {
            enable_stderr,
            level,
            log_path: None,
            max_file_size: 0,
            max_backup: 0,
        }
    }
}

pub struct LoggerHandle {
    arg: Mutex<LoggerConfigArg>,
    handle: Handle,
}

impl LoggerHandle {
    fn new(arg: LoggerConfigArg, handle: Handle) -> Self {
        Self {
            arg: Mutex::new(arg),
            handle,
        }
    }

    pub fn enable_stderr(&self) {
        let mut arg = self.arg.lock().unwrap().clone();
        arg.enable_stderr = true;
        self.update_logger(arg);
    }

    pub fn disable_stderr(&self) {
        let mut arg = self.arg.lock().unwrap().clone();
        arg.enable_stderr = false;
        self.update_logger(arg);
    }

    pub fn enable_file(&self, log_path: PathBuf, max_file_size: u64, max_backup: u32) {
        let mut arg = self.arg.lock().unwrap().clone();
        arg.log_path = Some(log_path);
        arg.max_file_size = max_file_size;
        arg.max_backup = max_backup;
        self.update_logger(arg);
    }

    pub fn update_level(&self, level: LevelFilter) {
        let mut arg = self.arg.lock().unwrap().clone();
        arg.level = level;
        self.update_logger(arg);
    }

    fn update_logger(&self, arg: LoggerConfigArg) {
        let mut origin_arg = self.arg.lock().unwrap();
        if *origin_arg != arg {
            let config = build_config(arg.clone()).expect("rebuild log config should success.");
            *origin_arg = arg;
            self.handle.set_config(config);
        }
    }

    /// Get log path
    pub fn log_path(&self) -> Option<PathBuf> {
        self.arg.lock().unwrap().log_path.as_ref().cloned()
    }

    /// Check is stderr enabled
    pub fn stderr(&self) -> bool {
        self.arg.lock().unwrap().enable_stderr
    }

    pub fn level(&self) -> LevelFilter {
        self.arg.lock().unwrap().level
    }
}

const LOG_PATTERN: &str = "{d} {l} {M}::{f}::{L} - {m}{n}";

fn build_config(arg: LoggerConfigArg) -> Result<Config> {
    let LoggerConfigArg {
        enable_stderr,
        level,
        log_path,
        max_file_size,
        max_backup,
    } = arg;
    if !enable_stderr && log_path.is_none() {
        println!("Logger is disabled.");
    }
    let mut builder = Config::builder();
    let mut root_builder = Root::builder();
    if enable_stderr {
        let stderr = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
            .target(Target::Stderr)
            .build();
        builder = builder.appender(Appender::builder().build("stderr", Box::new(stderr)));
        root_builder = root_builder.appender("stderr");
    }
    if let Some(log_path) = log_path {
        let log_file_backup_pattern =
            format!("{}.{{}}.gz", log_path.to_str().expect("invalid log_path"));
        let file_appender = RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
            .build(
                log_path,
                Box::new(CompoundPolicy::new(
                    Box::new(SizeTrigger::new(max_file_size)),
                    Box::new(
                        FixedWindowRoller::builder()
                            .build(log_file_backup_pattern.as_str(), max_backup)
                            .map_err(|e| format_err!("{:?}", e))?,
                    ),
                )),
            )
            .expect("build file logger fail.");

        builder = builder.appender(Appender::builder().build("file", Box::new(file_appender)));
        root_builder = root_builder.appender("file");
    }

    builder
        .build(root_builder.build(level))
        .map_err(|e| e.into())
}

fn env_log_level(default_level: &str) -> LevelFilter {
    let level = std::env::var("RUST_LOG").unwrap_or_else(|_| default_level.to_string());
    level
        .parse()
        .unwrap_or_else(|_| panic!("Unexpect log level: {}", level))
}

lazy_static! {
    static ref LOGGER_HANDLE: Mutex<Option<Arc<LoggerHandle>>> = Mutex::new(None);
}

pub fn init() -> Arc<LoggerHandle> {
    init_with_default_level("info")
}

pub fn init_with_default_level(default_level: &str) -> Arc<LoggerHandle> {
    let level = env_log_level(default_level);
    LOG_INIT.call_once(|| {
        let arg = LoggerConfigArg::new(true, level);
        let config = build_config(arg.clone()).expect("build log config fail.");
        let handle = match log4rs::init_config(config) {
            Ok(handle) => handle,
            Err(e) => panic!(e.to_string()),
        };
        let logger_handle = LoggerHandle::new(arg, handle);

        *LOGGER_HANDLE.lock().unwrap() = Some(Arc::new(logger_handle));
    });

    let logger_handle = LOGGER_HANDLE
        .lock()
        .unwrap()
        .as_ref()
        .expect("logger handle must has been set.")
        .clone();
    if logger_handle.level() != level {
        logger_handle.update_level(level);
    }
    logger_handle
}

static LOG_INIT: Once = Once::new();

pub fn init_for_test() -> Arc<LoggerHandle> {
    init_with_default_level("debug")
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_log() {
        let handle = super::init_for_test();
        debug!("debug message2.");
        info!("info message.");
        warn!("warn message.");
        error!("error message.");
        let handle2 = super::init_for_test();
        assert_eq!(handle.level(), handle2.level());
        assert_eq!(handle.log_path(), handle2.log_path());
        assert_eq!(handle.stderr(), handle2.stderr());
        let origin_level = handle.level();

        handle.update_level(LevelFilter::Off);

        assert_eq!(handle.level(), LevelFilter::Off);
        assert_eq!(handle.level(), handle2.level());

        handle.update_level(origin_level);
    }
}
