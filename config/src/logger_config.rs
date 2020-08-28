// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

static LOGGER_FILE_NAME: &str = "starcoin.log";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoggerConfig {
    pub enable_stderr: bool,
    pub enable_file: bool,
    pub max_file_size: u64,
    pub max_backup: u32,
    #[serde(skip)]
    log_path: Option<PathBuf>,
}

impl LoggerConfig {
    pub fn get_log_path(&self) -> PathBuf {
        self.log_path
            .as_ref()
            .expect("log path should init.")
            .clone()
    }

    pub fn enable_file(&self) -> bool {
        self.enable_file && self.log_path.is_some()
    }
}

impl ConfigModule for LoggerConfig {
    fn default_with_opt(opt: &StarcoinOpt, base: &BaseConfig) -> Result<Self> {
        let enable_stderr = !opt.disable_std_log;
        let enable_file = !opt.disable_file_log;

        Ok(if base.net.is_test() {
            Self {
                enable_stderr,
                enable_file,
                max_file_size: 10 * 1024 * 1024,
                max_backup: 1,
                log_path: None,
            }
        } else if base.net.is_dev() {
            Self {
                enable_stderr,
                enable_file,
                max_file_size: 10 * 1024 * 1024,
                max_backup: 2,
                log_path: None,
            }
        } else {
            Self {
                enable_stderr,
                enable_file,
                max_file_size: 1024 * 1024 * 1024,
                max_backup: 7,
                log_path: None,
            }
        })
    }

    fn after_load(&mut self, opt: &StarcoinOpt, base: &BaseConfig) -> Result<()> {
        self.log_path = Some(base.data_dir.join(LOGGER_FILE_NAME));
        self.enable_stderr = !opt.disable_std_log;
        self.enable_file = !opt.disable_file_log;
        Ok(())
    }
}
