// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
#[derive(Debug, Parser)]
#[clap(
    name = "Move Explain",
    about = "Explain Move abort codes. Errors are defined as a global category + module-specific reason for the error."
)]
struct Args {
    /// The location (module id) returned with a `MoveAbort` error
    #[clap(long = "location", short = 'l')]
    location: String,
    /// The abort code returned with a `MoveAbort` error
    #[clap(long = "abort-code", short = 'a')]
    abort_code: u64,
}

fn main() {
    let args = Args::parse();
    match starcoin_move_explain::get_explanation(&args.location, args.abort_code) {
        None => println!(
            "Unable to find a description for {}::{}",
            args.location, args.abort_code
        ),
        Some(error_desc) => println!(
            "Category:\n  Name: {}\n  Description: {}",
            error_desc.code_name, error_desc.code_description,
        ),
    }
}
