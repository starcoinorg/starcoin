// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;

static LOGGER_FILE_NAME: &str = "starcoin.log";

const DEFAULT_MAX_FILE_SIZE: u64 = 1024 * 1024 * 1024;
const MAX_FILE_SIZE_FOR_TEST: u64 = 10 * 1024 * 1024;
const DEFAULT_MAX_BACKUP: u32 = 7;

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct LoggerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "logger-disable-stderr", long, help = "disable stderr logger")]
    pub disable_stderr: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "logger-disable-file", long, help = "disable file logger")]
    pub disable_file: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "logger-max-file-size", long)]
    pub max_file_size: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(name = "logger-max-backup", long)]
    pub max_backup: Option<u32>,

    #[structopt(skip)]
    #[serde(skip)]
    base: Option<Arc<BaseConfig>>,
}

impl LoggerConfig {
    fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init.")
    }

    pub fn get_log_path(&self) -> Option<PathBuf> {
        if self.disable_file() {
            return None;
        }
        let log_path = self.base().data_dir.join(LOGGER_FILE_NAME);
        Some(log_path)
    }

    pub fn enable_file(&self) -> bool {
        !self.disable_file.unwrap_or(false)
    }

    pub fn disable_file(&self) -> bool {
        self.disable_file
            .unwrap_or_else(|| self.base().net().is_test())
    }

    pub fn disable_stderr(&self) -> bool {
        self.disable_stderr.unwrap_or(false)
    }

    pub fn max_file_size(&self) -> u64 {
        self.max_file_size.unwrap_or_else(|| {
            let base = self.base();
            if base.net().is_test() || base.net().is_dev() {
                MAX_FILE_SIZE_FOR_TEST
            } else {
                DEFAULT_MAX_FILE_SIZE
            }
        })
    }

    pub fn max_backup(&self) -> u32 {
        self.max_backup.unwrap_or_else(|| {
            let base = self.base();
            if base.net().is_test() {
                1
            } else if base.net().is_dev() {
                2
            } else {
                DEFAULT_MAX_BACKUP
            }
        })
    }
}

impl ConfigModule for LoggerConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, base: Arc<BaseConfig>) -> Result<()> {
        self.base = Some(base);
        if opt.logger.disable_stderr.is_some() {
            self.disable_stderr = opt.logger.disable_stderr;
        }
        if opt.logger.disable_file.is_some() {
            self.disable_file = opt.logger.disable_file;
        }
        if opt.logger.max_file_size.is_some() {
            self.max_file_size = opt.logger.max_file_size;
        }
        if opt.logger.max_backup.is_some() {
            self.max_backup = opt.logger.max_backup;
        }
        Ok(())
    }
}
