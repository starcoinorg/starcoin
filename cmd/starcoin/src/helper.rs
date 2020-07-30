// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_logger::prelude::*;
use std::path::Path;
use std::time::Duration;

//TODO use notify to implement.
//TODO move to a suitable crate
pub fn wait_until_file_created(file_path: &Path) -> Result<()> {
    let mut count = 0;
    loop {
        if count >= 20 {
            return Err(anyhow::anyhow!("wait file created timeout > 10s"));
        }
        debug!("Wait file {:?} create.", file_path);
        if !file_path.exists() {
            count += 1;
            std::thread::sleep(Duration::from_millis(500));
        } else {
            break;
        }
    }
    Ok(())
}
