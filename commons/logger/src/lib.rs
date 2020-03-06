// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use env_logger::Env;
pub use log::{debug, error, info, log, log_enabled, trace, warn, Level};

/// Logger prelude which includes all logging macros.
pub mod prelude {
    pub use log::{debug, error, info, log_enabled, trace, warn, Level};
}

pub fn init() {
    let env = Env::new().filter_or("RUST_LOG", "info");
    //env_logger::init();
    let _ = env_logger::from_env(env).try_init();
}

pub fn init_for_test() {
    let env = Env::new().filter_or("RUST_LOG", "debug");
    let _ = env_logger::from_env(env).is_test(true).try_init();
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
