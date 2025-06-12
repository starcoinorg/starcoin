// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod export;
mod import;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(
    name = "resource-code-exporter",
    about = "Export and import state data"
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Export state data to CSV file
    Export {
        #[clap(long, short = 'o', parse(from_os_str))]
        /// Output file path, e.g. accounts.csv
        output: PathBuf,

        #[clap(long, short = 'i', parse(from_os_str))]
        /// Starcoin node db path, e.g. ~/.starcoin/barnard/starcoindb/db/starcoindb
        db_path: PathBuf,

        #[clap(long)]
        /// Block id to export state at
        block_hash: starcoin_crypto::HashValue,

        #[clap(long, short = 's')]
        /// Start account index
        start: u64,

        #[clap(long, short = 'e')]
        /// End account index, 0 to process to end of data list
        end: u64,
    },
    /// Import state data from CSV file
    Import {
        #[clap(long, short = 'i', parse(from_os_str))]
        /// Input CSV file path
        csv_input: PathBuf,

        #[clap(long, short = 'd', parse(from_os_str))]
        /// Output database path
        db_path: PathBuf,

        #[clap(long)]
        /// expect state root hash
        state_root: starcoin_crypto::HashValue,

        #[clap(long, short = 's')]
        /// Start account index
        start: u64,

        #[clap(long, short = 'e')]
        /// End account index, 0 to process to end of data list
        end: u64,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    starcoin_logger::init();

    match cli.command {
        Commands::Export {
            output,
            db_path,
            block_hash,
            start,
            end,
        } => {
            index_check(start, end);
            export::export(
                db_path.display().to_string().as_str(),
                &output,
                block_hash,
                start,
                end,
            )?;
        }
        Commands::Import {
            csv_input,
            db_path,
            state_root,
            start,
            end,
        } => {
            index_check(start, end);
            import::import(&csv_input, &db_path, state_root, start, end)?;
        }
    }

    Ok(())
}

fn index_check(start: u64, end: u64) {
    assert!(start < end && end != 0);
}
