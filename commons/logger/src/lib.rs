// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use log::LevelFilter;
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Handle;
use std::cell::RefCell;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Once;

/// Logger prelude which includes all logging macros.
pub mod prelude {
    pub use log::{debug, error, info, log_enabled, trace, warn, Level};
}

#[derive(Clone, PartialOrd, PartialEq, Ord, Eq)]
struct LoggerConfigArg {
    enable_stderr: bool,
    level: LevelFilter,
    log_path: Option<PathBuf>,
}

impl LoggerConfigArg {
    pub fn new(enable_stderr: bool, level: LevelFilter, log_path: Option<PathBuf>) -> Self {
        Self {
            enable_stderr,
            level,
            log_path,
        }
    }
}

pub struct LoggerHandle {
    arg: RefCell<LoggerConfigArg>,
    handle: Handle,
}

impl LoggerHandle {
    fn new(
        enable_stderr: bool,
        level: LevelFilter,
        log_path: Option<PathBuf>,
        handle: Handle,
    ) -> Self {
        Self {
            arg: RefCell::new(LoggerConfigArg {
                enable_stderr,
                level,
                log_path,
            }),
            handle,
        }
    }

    pub fn enable_stderr(&self) {
        let mut arg = self.arg.borrow().clone();
        arg.enable_stderr = true;
        self.update_logger(arg);
    }

    pub fn disable_stderr(&self) {
        let mut arg = self.arg.borrow().clone();
        arg.enable_stderr = false;
        self.update_logger(arg);
    }

    pub fn enable_file(&self, enable_stderr: bool, log_path: PathBuf) {
        let mut arg = self.arg.borrow().clone();
        arg.enable_stderr = enable_stderr;
        arg.log_path = Some(log_path);
        self.update_logger(arg);
    }

    pub fn update_level(&self, level: &str) -> Result<()> {
        let level = level.parse()?;
        let mut arg = self.arg.borrow().clone();
        arg.level = level;
        self.update_logger(arg);
        Ok(())
    }

    fn update_logger(&self, arg: LoggerConfigArg) {
        if self.arg.borrow().clone() != arg {
            let config = build_config(arg.clone()).expect("rebuild log config should success.");
            self.arg.replace(arg);
            self.handle.set_config(config);
        }
    }

    pub fn log_path(&self) -> Option<PathBuf> {
        self.arg.borrow().log_path.as_ref().cloned()
    }
}

const LOG_PATTERN: &str = "{d} {l} {M}::{f}::{L} - {m}{n}";

fn build_config(arg: LoggerConfigArg) -> Result<Config> {
    let LoggerConfigArg {
        enable_stderr,
        level,
        log_path,
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
        let file_appender = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
            .build(log_path)
            .unwrap();

        builder = builder.appender(Appender::builder().build("file", Box::new(file_appender)));
        root_builder = root_builder.appender("file");
    }

    builder
        .build(root_builder.build(level))
        .map_err(|e| e.into())
}

fn env_log_level(default_level: &str) -> LevelFilter {
    let level = option_env!("RUST_LOG").unwrap_or(default_level);
    level
        .parse()
        .expect(format!("Unexpect log level: {}", level).as_str())
}

pub fn init() -> LoggerHandle {
    init_with_default_level("info")
}

pub fn init_with_default_level(default_level: &str) -> LoggerHandle {
    let level = env_log_level(default_level);
    let config =
        build_config(LoggerConfigArg::new(true, level, None)).expect("build log config fail.");
    let handle = match log4rs::init_config(config) {
        Ok(handle) => handle,
        Err(e) => panic!(format!("{}", e.description())),
    };
    LoggerHandle::new(true, level, None, handle)
}

static TEST_LOG_INIT: Once = Once::new();

pub fn init_for_test() {
    TEST_LOG_INIT.call_once(|| {
        let _ = init_with_default_level("debug");
    });
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_log() {
        super::init_for_test();
        debug!("debug message.");
        info!("info message.");
        warn!("warn message.");
        error!("error message.");
    }
}
