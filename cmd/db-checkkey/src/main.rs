// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::InnerStore;
use starcoin_storage::VEC_PREFIX_NAME;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "db-scan", about = "starcoin db scan")]
pub struct ScanOptions {
    #[structopt(long, short = "i", parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/barnard/starcoindb/db/starcoindb
    pub db_path: PathBuf,
    #[structopt(long, short = "n")]
    pub cf_name: String,
    #[structopt(long, short = "b")]
    pub block_hash: HashValue,
}

fn main() -> anyhow::Result<()> {
    println!("family {:?}", VEC_PREFIX_NAME.to_vec());
    let option = ScanOptions::from_args();
    let db = DBStorage::open_with_cfs(
        option.db_path.display().to_string().as_str(),
        VEC_PREFIX_NAME.to_vec(),
        true,
        Default::default(),
        None,
    )?;

    let result = db.get(option.cf_name.as_str(), option.block_hash.to_vec())?;
    if result.is_some() {
        println!("{} block_hash {} exist", option.cf_name, option.block_hash);
    } else {
        println!(
            "{} block_hash {} not exist",
            option.cf_name, option.block_hash
        );
    }
    Ok(())
}
