// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_logger::prelude::*;
use std::path::PathBuf;
use std::time::Duration;

//TODO use notify to implement.
//TODO move to a suitable crate
//TODO timeout.
pub fn wait_until_file_created(file_path: &PathBuf) -> Result<()> {
    loop {
        debug!("Wait file {:?} create.", file_path);
        if !file_path.exists() {
            std::thread::sleep(Duration::from_millis(500));
        } else {
            break;
        }
    }
    Ok(())
}
