// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use backtrace::Backtrace;
use starcoin_logger::prelude::*;
use std::{
    panic::{self, PanicInfo},
    process, thread, time,
};

/// Invoke to ensure process exits on a thread panic.
pub fn setup_panic_handler() {
    panic::set_hook(Box::new(move |pi: &PanicInfo<'_>| {
        handle_panic(pi);
    }));
}

// Formats and logs panic information
fn handle_panic(panic_info: &PanicInfo<'_>) {
    let details = format!("{}", panic_info);
    let backtrace = format!("{:#?}", Backtrace::new());

    error!("panic occurred:");
    println!("panic occurred:");
    error!("details: {}", details);
    println!("details: {}", details);
    error!("backtrace: {}", backtrace);
    println!("backtrace: {}", backtrace);

    // Provide some time to save the log to disk
    thread::sleep(time::Duration::from_millis(100));
    // Kill the process
    process::exit(12);
}
