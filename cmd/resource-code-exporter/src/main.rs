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
        block_id: starcoin_crypto::HashValue,
    },
    /// Import state data from CSV file
    Import {
        #[clap(long, short = 'i', parse(from_os_str))]
        /// Input CSV file path
        csv_input: PathBuf,

        #[clap(long, short = 'd', parse(from_os_str))]
        /// Output database path
        db_path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Export {
            output,
            db_path,
            block_id,
        } => {
            export::export(db_path.display().to_string().as_str(), &output, block_id)?;
        }
        Commands::Import { csv_input, db_path } => {
            import::import(&csv_input, &db_path)?;
        }
    }

    Ok(())
}
