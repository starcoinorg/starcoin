// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_unix_ts() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_nanos()
}
