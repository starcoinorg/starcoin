// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use arc_swap::ArcSwap;
use lazy_static::lazy_static;
use slog::{o, Discard, Drain, Logger};
use slog_async::Async;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const TIMESTAMP_FORMAT: &str = "%+";

// Defined global slog logger
lazy_static! {
    static ref GLOBAL_SLOG_LOGGER: ArcSwap<Logger> =
        ArcSwap::from(Arc::new(Logger::root(Discard, o!())));
}

// A RuntimeLevelFilter will discard all log records whose log level is less than the level
// specified in the struct.
pub struct RuntimeLevelFilter<D> {
    drain: D,
    level: Mutex<slog::Level>,
}

impl<D> RuntimeLevelFilter<D> {
    pub fn new(drain: D, level: slog::Level) -> Self {
        RuntimeLevelFilter {
            drain,
            level: Mutex::new(level),
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
    is_async: bool,
    chan_size: Option<usize>,
    log_path: PathBuf,
) -> Result<Logger> {
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(log_path)?;

    let decorator = slog_term::PlainDecorator::new(file);
    let drain = slog_term::CompactFormat::new(decorator)
        .use_custom_timestamp(timestamp_custom)
        .build()
        .fuse();

    if is_async {
        let async_builder = match chan_size {
            Some(chan_size_inner) => Async::new(drain).chan_size(chan_size_inner),
            None => Async::new(drain),
        };
        Ok(Logger::root(async_builder.build().fuse(), o!()))
    } else {
        Ok(Logger::root(Mutex::new(drain).fuse(), o!()))
    }
}

pub fn set_global_logger(is_async: bool, chan_size: Option<usize>, file: PathBuf) -> Result<()> {
    let logger = create_default_root_logger(is_async, chan_size, file)?;
    GLOBAL_SLOG_LOGGER.store(Arc::new(logger));
    Ok(())
}

pub fn with_logger<F, R>(f: F) -> R
where
    F: FnOnce(&Logger) -> R,
{
    f(&(*GLOBAL_SLOG_LOGGER.load()))
}
