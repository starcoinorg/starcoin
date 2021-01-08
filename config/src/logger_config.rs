// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

static LOGGER_FILE_NAME: &str = "starcoin.log";

const DEFAULT_MAX_FILE_SIZE: u64 = 1024 * 1024 * 1024;
const MAX_FILE_SIZE_FOR_TEST: u64 = 10 * 1024 * 1024;
const DEFAULT_MAX_BACKUP: u32 = 7;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct LoggerConfig {
    #[structopt(name = "disable-stderr", long, help = "disable stderr logger")]
    pub disable_stderr: Option<bool>,
    #[structopt(name = "disable-file", long, help = "disable file logger")]
    pub disable_file: Option<bool>,
    #[structopt(name = "max-file-size", long, default_value = "1073741824")]
    pub max_file_size: u64,
    #[structopt(name = "max-backup", long, default_value = "7")]
    pub max_backup: u32,
    #[structopt(skip)]
    #[serde(skip)]
    log_path: Option<PathBuf>,
}
impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            disable_stderr: None,
            disable_file: None,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            max_backup: DEFAULT_MAX_BACKUP,
            log_path: None,
        }
    }
}
impl LoggerConfig {
    pub fn get_log_path(&self) -> PathBuf {
        self.log_path
            .as_ref()
            .expect("log path should init.")
            .clone()
    }

    pub fn enable_file(&self) -> bool {
        let disable = self.disable_file.unwrap_or(false);
        (!disable) && self.log_path.is_some()
    }

    pub fn disable_stderr(&self) -> bool {
        self.disable_stderr.unwrap_or(false)
    }
}

impl ConfigModule for LoggerConfig {
    fn default_with_opt(opt: &StarcoinOpt, base: &BaseConfig) -> Result<Self> {
        let disable_stderr = opt.logger.disable_stderr;
        let disable_file = opt.logger.disable_file;
        Ok(if base.net.is_test() {
            Self {
                disable_stderr,
                disable_file,
                max_file_size: MAX_FILE_SIZE_FOR_TEST,
                max_backup: 1,
                log_path: None,
            }
        } else if base.net.is_dev() {
            Self {
                disable_stderr,
                disable_file,
                max_file_size: MAX_FILE_SIZE_FOR_TEST,
                max_backup: 2,
                log_path: None,
            }
        } else {
            Self {
                disable_stderr,
                disable_file,
                max_file_size: DEFAULT_MAX_FILE_SIZE,
                max_backup: DEFAULT_MAX_BACKUP,
                log_path: None,
            }
        })
    }

    fn after_load(&mut self, opt: &StarcoinOpt, base: &BaseConfig) -> Result<()> {
        self.log_path = Some(base.data_dir.join(LOGGER_FILE_NAME));
        if opt.logger.disable_stderr.is_some() {
            self.disable_stderr = opt.logger.disable_stderr;
        }
        if opt.logger.disable_file.is_some() {
            self.disable_file = opt.logger.disable_file;
        }
        Ok(())
    }
}
