// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod export;
mod import;

use crate::export::ExporterOptions;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let option: ExporterOptions = ExporterOptions::parse();
    let output = option.output.as_path();
    let block_id = option.block_id;
    export::export(
        option.db_path.display().to_string().as_str(),
        output,
        block_id,
    )?;
    Ok(())
}
