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
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
    Handle,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;
use std::sync::{Arc, Once};

/// Logger prelude which includes all logging macros.
pub mod prelude {
    pub use log::{debug, error, info, log_enabled, trace, warn, Level, LevelFilter};
}

const LOG_PATTERN_WITH_LINE: &str = "{d} {l} {M}::{f}::{L} - {m}{n}";
const LOG_PATTERN_DEFAULT: &str = "{d} {l} - {m}{n}";

#[derive(Clone, Debug, Hash, PartialOrd, PartialEq, Ord, Eq, Serialize, Deserialize)]
pub enum LogPattern {
    Default,
    WithLine,
    Custom(String),
}

impl FromStr for LogPattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "default" => LogPattern::Default,
            "withline" | "with_line" => LogPattern::WithLine,
            _ => LogPattern::Custom(s.to_owned()),
        })
    }
}

impl LogPattern {
    pub fn get_pattern(&self) -> String {
        match self {
            LogPattern::Default => LOG_PATTERN_DEFAULT.to_owned(),
            LogPattern::WithLine => LOG_PATTERN_WITH_LINE.to_owned(),
            LogPattern::Custom(pattern) => pattern.clone(),
        }
    }

    pub fn by_level(level: LevelFilter) -> LogPattern {
        match level {
            LevelFilter::Trace | LevelFilter::Debug => LogPattern::WithLine,
            _ => LogPattern::Default,
        }
    }
}

impl std::fmt::Display for LogPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_str = match self {
            LogPattern::Default => "default".to_owned(),
            LogPattern::WithLine => "withline".to_owned(),
            LogPattern::Custom(p) => format!("custom({})", p),
        };
        write!(f, "{}", display_str)
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]
struct LoggerConfigArg {
    enable_stderr: bool,
    // global level
    level: LevelFilter,
    // sub path level
    module_levels: Vec<(String, LevelFilter)>,
    log_path: Option<PathBuf>,
    max_file_size: u64,
    max_backup: u32,
    pattern: LogPattern,
}

impl LoggerConfigArg {
    pub fn new(
        enable_stderr: bool,
        level: LevelFilter,
        module_levels: Vec<(String, LevelFilter)>,
        pattern: LogPattern,
    ) -> Self {
        Self {
            enable_stderr,
            level,
            module_levels,
            log_path: None,
            max_file_size: 0,
            max_backup: 0,
            pattern,
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
        arg.pattern = LogPattern::by_level(level);
        self.update_logger(arg);
    }

    pub fn set_log_level(&self, logger_name: String, level: LevelFilter) {
        let mut arg = self.arg.lock().unwrap().clone();
        if let Some(t) = arg
            .module_levels
            .iter_mut()
            .find(|(n, _)| n == &logger_name)
        {
            t.1 = level;
        } else {
            arg.module_levels.push((logger_name, level));
        }
        self.update_logger(arg);
    }

    pub fn set_log_pattern(&self, pattern: LogPattern) {
        let mut arg = self.arg.lock().unwrap().clone();
        arg.pattern = pattern;
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

fn build_config(arg: LoggerConfigArg) -> Result<Config> {
    let LoggerConfigArg {
        enable_stderr,
        level,
        module_levels,
        log_path,
        max_file_size,
        max_backup,
        pattern,
    } = arg;
    if !enable_stderr && log_path.is_none() {
        println!("Logger is disabled.");
    }
    let mut builder = Config::builder();
    let mut root_builder = Root::builder();
    if enable_stderr {
        let stderr = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                pattern.get_pattern().as_str(),
            )))
            .target(Target::Stderr)
            .build();
        builder = builder.appender(Appender::builder().build("stderr", Box::new(stderr)));
        root_builder = root_builder.appender("stderr");
    }
    if let Some(log_path) = log_path {
        let log_file_backup_pattern =
            format!("{}.{{}}.gz", log_path.to_str().expect("invalid log_path"));
        let file_appender = RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                pattern.get_pattern().as_str(),
            )))
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

    builder = builder.loggers(
        module_levels
            .into_iter()
            .map(|(name, level)| Logger::builder().build(name, level)),
    );

    builder
        .build(root_builder.build(level))
        .map_err(|e| e.into())
}

/// read log level filters from `RUST_LOG` env.
/// return global level filter and specified level filters.
fn env_log_level(default_level: &str) -> (LevelFilter, Vec<(String, LevelFilter)>) {
    let level_str = std::env::var("RUST_LOG").unwrap_or_default();
    let level_filters = parse_spec(level_str.as_str());
    let default_level = level_filters.global_level.unwrap_or_else(|| {
        default_level
            .parse()
            .unwrap_or_else(|_| panic!("Unexpect log level: {}", default_level))
    });
    (default_level, level_filters.module_levels)
}

lazy_static! {
    static ref LOGGER_HANDLE: Mutex<Option<Arc<LoggerHandle>>> = Mutex::new(None);
}

pub fn init() -> Arc<LoggerHandle> {
    init_with_default_level("info", LogPattern::Default)
}

pub fn init_with_default_level(default_level: &str, pattern: LogPattern) -> Arc<LoggerHandle> {
    let (global_level, module_levels) = env_log_level(default_level);
    LOG_INIT.call_once(|| {
        let arg = LoggerConfigArg::new(true, global_level, module_levels, pattern);
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
    if logger_handle.level() != global_level {
        logger_handle.update_level(global_level);
    }
    logger_handle
}

static LOG_INIT: Once = Once::new();

pub fn init_for_test() -> Arc<LoggerHandle> {
    init_with_default_level("debug", LogPattern::WithLine)
}

#[derive(Default, Eq, PartialEq, Clone, Debug)]
struct LogLevelSpec {
    global_level: Option<LevelFilter>,
    module_levels: Vec<(String, LevelFilter)>,
}

/// Parse a logging specification string (e.g: "crate1,crate2::mod3,crate3::x=error/foo")
/// and return a vector with log filters.
fn parse_spec(spec: &str) -> LogLevelSpec {
    let mut parts = spec.split('/');
    let mods = parts.next();
    let filter = parts.next();
    if parts.next().is_some() {
        eprintln!(
            "warning: invalid logging spec '{}', \
             ignoring it (too many '/'s)",
            spec
        );
        return LogLevelSpec::default();
    }
    let mut dirs = Vec::new();
    let mut global_fallback_level: Option<LevelFilter> = None;
    if let Some(m) = mods {
        for s in m.split(',') {
            if s.is_empty() {
                continue;
            }
            let mut parts = s.split('=');
            let (log_level, name) =
                match (parts.next(), parts.next().map(|s| s.trim()), parts.next()) {
                    (Some(part0), None, None) => {
                        // if the single argument is a log-level string or number,
                        // treat that as a global fallback
                        match part0.parse() {
                            Ok(num) => {
                                if global_fallback_level.is_some() {
                                    eprintln!(
                                    "warning: multi global level specified, only use first one!"
                                );
                                } else {
                                    global_fallback_level = Some(num);
                                }
                                continue;
                            }
                            Err(_) => (LevelFilter::max(), part0),
                        }
                    }
                    (Some(part0), Some(""), None) => (LevelFilter::max(), part0),
                    (Some(part0), Some(part1), None) => match part1.parse() {
                        Ok(num) => (num, part0),
                        _ => {
                            eprintln!(
                                "warning: invalid logging spec '{}', \
                                 ignoring it",
                                part1
                            );
                            continue;
                        }
                    },
                    _ => {
                        eprintln!(
                            "warning: invalid logging spec '{}', \
                             ignoring it",
                            s
                        );
                        continue;
                    }
                };
            dirs.push((name.to_string(), log_level));
        }
    }
    if filter.is_some() {
        eprintln!("warning: filter by regexp is not supported yet");
    }
    // let filter = filter.map_or(None, |filter| match inner::Filter::new(filter) {
    //     Ok(re) => Some(re),
    //     Err(e) => {
    //         eprintln!("warning: invalid regex filter - {}", e);
    //         None
    //     }
    // });

    LogLevelSpec {
        global_level: global_fallback_level,
        module_levels: dirs,
    }
}

#[cfg(test)]
mod tests {
    use super::prelude::*;
    use crate::LogLevelSpec;

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

    #[test]
    fn test_log_level_spec() {
        let test_cases = vec![
            ("", LogLevelSpec::default()),
            (
                "info",
                LogLevelSpec {
                    global_level: Some(LevelFilter::Info),
                    module_levels: vec![],
                },
            ),
            (
                "debug,common=info,network=warn",
                LogLevelSpec {
                    global_level: Some(LevelFilter::Debug),
                    module_levels: vec![
                        ("common".to_string(), LevelFilter::Info),
                        ("network".to_string(), LevelFilter::Warn),
                    ],
                },
            ),
        ];

        for (spec_str, expected) in test_cases {
            let actual = super::parse_spec(spec_str);
            assert_eq!(actual, expected);
        }
    }
}
