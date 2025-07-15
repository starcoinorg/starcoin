// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod export;
mod import;

use clap::{Parser, Subcommand};
use starcoin_types::account_address::AccountAddress;
use std::fs;
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

        #[clap(long)]
        /// Whitelist file path containing account addresses (one per line)
        whitelist_file: Option<PathBuf>,

        #[clap(long, multiple_values = true)]
        /// Account addresses to export (can be specified multiple times)
        addresses: Vec<String>,
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
        state_root: starcoin_crypto::HashValue,
    },
}

fn parse_account_addresses(addresses: &[String]) -> anyhow::Result<Vec<AccountAddress>> {
    let mut parsed_addresses = Vec::new();
    for addr_str in addresses {
        let addr = addr_str.parse::<AccountAddress>()?;
        parsed_addresses.push(addr);
    }
    Ok(parsed_addresses)
}

fn read_whitelist_file(file_path: &PathBuf) -> anyhow::Result<Vec<AccountAddress>> {
    let content = fs::read_to_string(file_path)?;
    let mut addresses = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            let addr = line.parse::<AccountAddress>()?;
            addresses.push(addr);
        }
    }

    Ok(addresses)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    starcoin_logger::init();

    match cli.command {
        Commands::Export {
            output,
            db_path,
            block_hash,
            whitelist_file,
            addresses,
        } => {
            let mut white_list = None;

            if !addresses.is_empty() {
                let parsed_addresses = parse_account_addresses(&addresses)?;
                white_list = Some(parsed_addresses);
                println!(
                    "Using {} addresses from command line arguments",
                    addresses.len()
                );
            } else if let Some(whitelist_file_path) = whitelist_file {
                let file_addresses = read_whitelist_file(&whitelist_file_path)?;
                white_list = Some(file_addresses);
                println!(
                    "Using {} addresses from whitelist file: {}",
                    white_list.as_ref().unwrap().len(),
                    whitelist_file_path.display()
                );
            } else {
                println!("No whitelist provided, will export all accounts");
            }

            export::export(
                db_path.display().to_string().as_str(),
                &output,
                block_hash,
                white_list,
            )?;
        }
        Commands::Import {
            bcs_input,
            db_path,
            state_root,
        } => {
            import::import(&bcs_input, &db_path, state_root)?;
        }
    }

    Ok(())
}
