// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod export;
mod import;

use clap::{Parser, Subcommand};
use resource_code_exporter::uncle_search;
use starcoin_crypto::HashValue;
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
    },
    /// Import state data from BCS file
    Import {
        #[clap(long, short = 'i', parse(from_os_str))]
        /// Input BCS file path
        bcs_input: PathBuf,

        #[clap(long, short = 'd', parse(from_os_str))]
        /// Output database path
        db_path: PathBuf,

        #[clap(long)]
        /// expect state root hash
        state_root: HashValue,
    },
    /// Import state data from BCS file
    UncleSearch {
        #[clap(long, short = 'd', parse(from_os_str))]
        /// Output database path
        db_path: PathBuf,

        #[clap(long)]
        /// block hash
        block_hash: HashValue,

        #[clap(long, short = 'c')]
        /// Max size
        max_size: Option<usize>,
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
        } => {
            export::export(db_path.display().to_string().as_str(), &output, block_hash)?;
        }
        Commands::Import {
            bcs_input,
            db_path,
            state_root,
        } => {
            import::import(&bcs_input, &db_path, state_root)?;
        }
        Commands::UncleSearch {
            db_path,
            block_hash,
            max_size,
        } => {
            uncle_search::run(&db_path, block_hash, max_size)?;
        }
    }

    Ok(())
}
