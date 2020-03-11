// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::DEFAULT_DATA_DIR;
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StorageConfig {
    pub dir: PathBuf,
    #[serde(skip)]
    data_dir: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
          dir: PathBuf::from("starcoindb/db"),
          data_dir: (&*DEFAULT_DATA_DIR).clone()
        }
    }
}
impl StorageConfig {
    pub fn dir(&self) -> PathBuf {
        if self.dir.is_relative() {
            self.data_dir.join(&self.dir)
        } else {
            self.dir.clone()
        }
    }

    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        self.data_dir = data_dir;
    }
}