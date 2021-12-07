// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::structured_log::disable_slog_stderr;
use anyhow::{format_err, Result};
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
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Once};

pub mod structured_log;

/// Logger prelude which includes all logging macros.
pub mod prelude {
    pub use crate::stacktrace;
    pub use crate::{sl_crit, sl_debug, sl_error, sl_info, sl_trace, sl_warn};
    pub use log::{debug, error, info, log, log_enabled, trace, warn, Level, LevelFilter};
    pub use slog::{slog_crit, slog_debug, slog_error, slog_info, slog_trace, slog_warn};
}

pub fn stacktrace(err: anyhow::Error) {
    for cause in err.chain() {
        log::error!("{:?}", cause);
    }
}

const LOG_PATTERN_WITH_LINE: &str = "{d} {l} {M}::{f}::{L} - {m}{n}";
const LOG_PATTERN_DEFAULT: &str = "{d} {l} - {m}{n}";

#[derive(
    Clone, Debug, Hash, PartialOrd, PartialEq, Ord, Eq, Serialize, Deserialize, JsonSchema,
)]
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

/// set some third party module's default log level for reduce debug log.
static THIRD_PARTY_MODULES: Lazy<Vec<(&str, LevelFilter)>> = Lazy::new(|| {
    vec![
        // ("tokio_reactor", LevelFilter::Info),
        // ("yamux", LevelFilter::Info),
        // ("jsonrpc_core", LevelFilter::Info),
        // ("jsonrpc_client_transports", LevelFilter::Info),
        // ("parity_ws", LevelFilter::Info),
        // ("multistream_select", LevelFilter::Info),
        //("network_p2p", LevelFilter::Info),
        //sub-libp2p and sync is network_p2p log target.
        //("sub-libp2p", LevelFilter::Info),
        //("sync", LevelFilter::Info),
        // ("libp2p", LevelFilter::Info),
        // ("libp2p_swarm", LevelFilter::Info),
        // ("libp2p_core", LevelFilter::Info),
        // ("libp2p_ping", LevelFilter::Info),
        // ("libp2p_websocket", LevelFilter::Info),
        // ("libp2p_tcp", LevelFilter::Info),
        // ("libp2p_noise", LevelFilter::Info),
        // ("libp2p_dns", LevelFilter::Info),
        // ("libp2p_identify", LevelFilter::Info),
        // ("jsonrpc_ws_server", LevelFilter::Info),
        // ("jsonrpc_core", LevelFilter::Info),
        // ("rustyline", LevelFilter::Warn),
    ]
});

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoggerConfigArg {
    enable_stderr: bool,
    // global level
    level: LevelFilter,
    // sub path level
    module_levels: HashMap<String, LevelFilter>,
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
        pattern: Option<LogPattern>,
    ) -> Self {
        let mut default_module_levels = THIRD_PARTY_MODULES
            .iter()
            .map(|(m, m_level)| {
                (
                    m.to_string(),
                    if &level <= m_level { level } else { *m_level },
                )
            })
            .collect::<HashMap<_, _>>();
        default_module_levels.extend(module_levels);
        Self {
            enable_stderr,
            level,
            module_levels: default_module_levels,
            log_path: None,
            max_file_size: 0,
            max_backup: 0,
            pattern: pattern.unwrap_or_else(|| LogPattern::by_level(level)),
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
        let mut arg = self.arg.lock().clone();
        arg.enable_stderr = true;
        self.update_logger(arg);
    }

    pub fn disable_stderr(&self) {
        let mut arg = self.arg.lock().clone();
        arg.enable_stderr = false;
        self.update_logger(arg.clone());
        if let Some(path) = arg.log_path {
            disable_slog_stderr(path);
        }
    }
    pub fn enable_file(&self, log_path: PathBuf, max_file_size: u64, max_backup: u32) {
        let mut arg = self.arg.lock().clone();
        arg.log_path = Some(log_path);
        arg.max_file_size = max_file_size;
        arg.max_backup = max_backup;
        self.update_logger(arg);
    }

    pub fn update_level(&self, level: LevelFilter) {
        let mut arg = self.arg.lock().clone();
        arg.level = level;
        arg.pattern = LogPattern::by_level(level);
        self.update_logger(arg);
    }

    pub fn set_log_level(&self, logger_name: String, level: LevelFilter) {
        let mut arg = self.arg.lock().clone();
        arg.module_levels.insert(logger_name, level);
        self.update_logger(arg);
    }

    pub fn set_log_pattern(&self, pattern: LogPattern) {
        let mut arg = self.arg.lock().clone();
        arg.pattern = pattern;
        self.update_logger(arg);
    }

    fn update_logger(&self, arg: LoggerConfigArg) {
        let mut origin_arg = self.arg.lock();
        if *origin_arg != arg {
            let config = build_config(arg.clone()).expect("rebuild log config should success.");
            *origin_arg = arg;
            self.handle.set_config(config);
        }
    }

    /// Get log path
    pub fn log_path(&self) -> Option<PathBuf> {
        self.arg.lock().log_path.as_ref().cloned()
    }

    /// Check is stderr enabled
    pub fn stderr(&self) -> bool {
        self.arg.lock().enable_stderr
    }

    pub fn level(&self) -> LevelFilter {
        self.arg.lock().level
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
        ..
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
        let appender =
            rolling_file_append("log_file", max_file_size, max_backup, pattern, log_path)?;
        builder = builder.appender(appender);
        root_builder = root_builder.appender("log_file");
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

fn rolling_file_append(
    append_name: &str,
    max_file_size: u64,
    max_backup: u32,
    pattern: LogPattern,
    log_path: PathBuf,
) -> Result<Appender> {
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
    Ok(Appender::builder().build(append_name, Box::new(file_appender)))
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

static LOGGER_HANDLE: Lazy<Mutex<Option<Arc<LoggerHandle>>>> = Lazy::new(|| Mutex::new(None));

pub fn init() -> Arc<LoggerHandle> {
    init_with_default_level("info", None)
}

pub fn init_with_default_level(
    default_level: &str,
    pattern: Option<LogPattern>,
) -> Arc<LoggerHandle> {
    let (global_level, module_levels) = env_log_level(default_level);
    LOG_INIT.call_once(|| {
        let arg = LoggerConfigArg::new(true, global_level, module_levels, pattern);
        let config = build_config(arg.clone()).expect("build log config fail.");
        let handle = match log4rs::init_config(config) {
            Ok(handle) => handle,
            Err(e) => panic!("{}", e.to_string()),
        };
        let logger_handle = LoggerHandle::new(arg, handle);
        *LOGGER_HANDLE.lock() = Some(Arc::new(logger_handle));
    });
    let logger_handle = LOGGER_HANDLE
        .lock()
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
    init_with_default_level("debug", Some(LogPattern::WithLine))
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
    LogLevelSpec {
        global_level: global_fallback_level,
        module_levels: dirs,
    }
}

/// Log a critical level message using current logger
#[macro_export]
macro_rules! sl_crit( ($($args:tt)+) => {
    $crate::structured_log::with_logger(|logger| slog_crit![logger, $($args)+])
};);
/// Log a error level message using current logger
#[macro_export]
macro_rules! sl_error( ($($args:tt)+) => {
    $crate::structured_log::with_logger(|logger| slog_error![logger, $($args)+])
};);
/// Log a warning level message using current logger
#[macro_export]
macro_rules! sl_warn( ($($args:tt)+) => {
    $crate::structured_log::with_logger(|logger| slog_warn![logger, $($args)+])
};);
/// Log a info level message using current logger
#[macro_export]
macro_rules! sl_info( ($($args:tt)+) => {
    $crate::structured_log::with_logger(|logger| slog_info![logger, $($args)+])
};);
/// Log a debug level message using current logger
#[macro_export]
macro_rules! sl_debug( ($($args:tt)+) => {
    $crate::structured_log::with_logger(|logger| slog_debug![logger, $($args)+])
};);
/// Log a trace level message using current logger
#[macro_export]
macro_rules! sl_trace( ($($args:tt)+) => {
    $crate::structured_log::with_logger(|logger| slog_trace![logger, $($args)+])
};);

#[cfg(test)]
mod tests;
