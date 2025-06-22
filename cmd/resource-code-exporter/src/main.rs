// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod export;
mod import;

use clap::{Parser, Subcommand};
use starcoin_types::account_address::AccountAddress;
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

        #[clap(long)]
        /// Comma-separated list of account addresses to include (whitelist). If not specified, all accounts will be processed.
        /// Example: 0x1,0x2,0x3
        white_list: Option<String>,
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
            white_list,
        } => {
            index_check(start, end);
            let parsed_white_list = parse_white_list(white_list)?;
            export::export(
                db_path.display().to_string().as_str(),
                &output,
                block_hash,
                start,
                end,
                parsed_white_list,
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

/// Parse comma-separated account addresses string into Vec<AccountAddress>
fn parse_white_list(white_list: Option<String>) -> anyhow::Result<Option<Vec<AccountAddress>>> {
    use std::str::FromStr;
    match white_list {
        None => Ok(None),
        Some(addr_str) => {
            if addr_str.trim().is_empty() {
                return Ok(None);
            }
            let addresses: Result<Vec<AccountAddress>, _> = addr_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| {
                    let s = if s.starts_with("0x") {
                        s.to_string()
                    } else {
                        format!("0x{}", s)
                    };
                    AccountAddress::from_str(&s)
                })
                .collect();
            match addresses {
                Ok(addrs) => {
                    if addrs.is_empty() {
                        Ok(None)
                    } else {
                        println!("Parsed {} whitelist addresses: {:?}", addrs.len(), addrs);
                        Ok(Some(addrs))
                    }
                }
                Err(e) => Err(anyhow::anyhow!(
                    "Failed to parse whitelist addresses: {}",
                    e
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_parse_white_list() {
        // Test None input
        assert!(parse_white_list(None).unwrap().is_none());

        // Test empty string
        assert!(parse_white_list(Some("".to_string())).unwrap().is_none());
        assert!(parse_white_list(Some("   ".to_string())).unwrap().is_none());

        // Test single address with 0x prefix
        let result = parse_white_list(Some("0x1".to_string())).unwrap();
        assert!(result.is_some());
        let addresses = result.unwrap();
        assert_eq!(addresses.len(), 1);
        assert_eq!(addresses[0], AccountAddress::from_str("0x1").unwrap());

        // Test single address without 0x prefix (should be auto prepended)
        let result = parse_white_list(Some("1".to_string())).unwrap();
        assert!(result.is_some());
        let addresses = result.unwrap();
        assert_eq!(addresses.len(), 1);
        assert_eq!(addresses[0], AccountAddress::from_str("0x1").unwrap());

        // Test multiple addresses with spaces
        let result = parse_white_list(Some("0x1, 0x2, 0x3".to_string())).unwrap();
        assert!(result.is_some());
        let addresses = result.unwrap();
        assert_eq!(addresses.len(), 3);
        assert_eq!(addresses[0], AccountAddress::from_str("0x1").unwrap());
        assert_eq!(addresses[1], AccountAddress::from_str("0x2").unwrap());
        assert_eq!(addresses[2], AccountAddress::from_str("0x3").unwrap());

        // Test mixed format (with and without 0x)
        let result = parse_white_list(Some("0x1,2,0x3".to_string())).unwrap();
        assert!(result.is_some());
        let addresses = result.unwrap();
        assert_eq!(addresses.len(), 3);
        assert_eq!(addresses[0], AccountAddress::from_str("0x1").unwrap());
        assert_eq!(addresses[1], AccountAddress::from_str("0x2").unwrap());
        assert_eq!(addresses[2], AccountAddress::from_str("0x3").unwrap());

        // Test with empty elements (should be filtered out)
        let result = parse_white_list(Some("0x1,,0x2, ,0x3".to_string())).unwrap();
        assert!(result.is_some());
        let addresses = result.unwrap();
        assert_eq!(addresses.len(), 3);
        assert_eq!(addresses[0], AccountAddress::from_str("0x1").unwrap());
        assert_eq!(addresses[1], AccountAddress::from_str("0x2").unwrap());
        assert_eq!(addresses[2], AccountAddress::from_str("0x3").unwrap());

        // Test invalid address (should return error)
        let result = parse_white_list(Some("0xinvalid".to_string()));
        assert!(result.is_err());
    }
}
