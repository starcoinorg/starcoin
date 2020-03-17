// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use chrono::Local;
use env_logger::fmt::Formatter;
use env_logger::Env;
use log::Record;
use std::io::Write;

/// Logger prelude which includes all logging macros.
pub mod prelude {
    pub use log::{debug, error, info, log_enabled, trace, warn, Level};
}

fn log_format(buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
    writeln!(
        buf,
        "{} {} [{}:{}] {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        record.level(),
        record.module_path().unwrap_or("<unnamed>"),
        record.line().unwrap_or(0),
        &record.args()
    )
}

pub fn init() {
    let env = Env::new().filter_or("RUST_LOG", "info");
    //env_logger::init();
    let _ = env_logger::from_env(env).format(log_format).try_init();
}

pub fn init_for_test() {
    let env = Env::new().filter_or("RUST_LOG", "debug");
    let _ = env_logger::from_env(env)
        .format(log_format)
        .is_test(true)
        .try_init();
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
