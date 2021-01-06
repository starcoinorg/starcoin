// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use systemstat::{Platform, System};

pub fn get_free_mem_size() -> Result<u64> {
    let sys = System::new();
    let free = match sys.memory() {
        Ok(mem) => mem.free.as_u64(),
        Err(_x) => 0u64,
    };
    Ok(free)
}
