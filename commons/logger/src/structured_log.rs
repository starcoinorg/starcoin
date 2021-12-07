// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use arc_swap::ArcSwap;
use lazy_static::lazy_static;
use slog::{o, Discard, Drain, Logger};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

const TIMESTAMP_FORMAT: &str = "%+";

// Defined global slog logger
lazy_static! {
    static ref GLOBAL_SLOG_LOGGER: ArcSwap<Logger> =
        ArcSwap::from(Arc::new(Logger::root(Discard, o!())));
    static ref SLOG_LEVEL: Arc<Mutex<slog::Level>> = Arc::new(Mutex::new(slog::Level::Info));
}

// A RuntimeLevelFilter will discard all log records whose log level is less than the level
// specified in the struct.
pub struct RuntimeLevelFilter<D> {
    drain: D,
    level: Arc<Mutex<slog::Level>>,
}

impl<D> RuntimeLevelFilter<D> {
    pub fn new(drain: D, level: slog::Level) -> Self {
        let mut s = SLOG_LEVEL.lock().unwrap();
        *s = level;
        RuntimeLevelFilter {
            drain,
            level: SLOG_LEVEL.clone(),
        }
    }
}

impl<D> Drain for RuntimeLevelFilter<D>
where
    D: Drain,
{
    type Ok = Option<D::Ok>;
    type Err = Option<D::Err>;

    fn log(
        &self,
        record: &slog::Record,
        values: &slog::OwnedKVList,
    ) -> std::result::Result<Self::Ok, Self::Err> {
        let log_level = self.level.lock().unwrap();
        if record.level().is_at_least(*log_level) {
            self.drain.log(record, values)?;
        }
        Ok(None)
    }
}

fn timestamp_custom(io: &mut dyn Write) -> std::io::Result<()> {
    write!(io, "{}", chrono::Local::now().format(TIMESTAMP_FORMAT))
}

/// Creates a root logger with config settings.
fn create_default_root_logger(
    log_path: PathBuf,
    level: slog::Level,
    enable_stderr: bool,
) -> Result<Logger> {
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(log_path)?;
    let decorator = slog_term::PlainDecorator::new(file);
    let file_drain = slog_term::CompactFormat::new(decorator)
        .use_custom_timestamp(timestamp_custom)
        .build();
    let decorator = slog_term::TermDecorator::new().build();
    if enable_stderr {
        let io_drain = slog_term::CompactFormat::new(decorator).build();
        let drain =
            RuntimeLevelFilter::new(slog::Duplicate::new(file_drain, io_drain), level).fuse();
        Ok(Logger::root(Mutex::new(drain).fuse(), o!()))
    } else {
        let drain = RuntimeLevelFilter::new(file_drain, level).fuse();
        Ok(Logger::root(Mutex::new(drain).fuse(), o!()))
    }
}

pub fn init_slog_logger(file: PathBuf, enable_stderr: bool) -> Result<()> {
    let logger = create_default_root_logger(file, slog::Level::Info, enable_stderr)?;
    GLOBAL_SLOG_LOGGER.store(Arc::new(logger));
    Ok(())
}

pub fn set_slog_level(level: &str) {
    let level = slog::Level::from_str(level).unwrap_or(slog::Level::Info);
    let mut slog_level = SLOG_LEVEL.lock().unwrap();
    *slog_level = level;
}

pub fn disable_slog_stderr(log_path: PathBuf) {
    match create_default_root_logger(log_path, slog::Level::Info, false) {
        Ok(logger) => {
            GLOBAL_SLOG_LOGGER.swap(Arc::new(logger));
        }
        Err(e) => log::warn!("Failed to disable slog stderr:{}", e),
    };
}

pub fn with_logger<F, R>(f: F) -> R
where
    F: FnOnce(&Logger) -> R,
{
    f(&(*GLOBAL_SLOG_LOGGER.load()))
}
