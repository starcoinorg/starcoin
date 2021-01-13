// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use network_api::messages::{CompactBlockMessage, TransactionsMessage};
use scs::{SCSCodec, Sample};
use serde::de::DeserializeOwned;
use serde::Serialize;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_types::block::{Block, BlockHeader, BlockInfo};
use starcoin_types::cmpact_block::CompactBlock;
use starcoin_types::startup_info::ChainStatus;
use starcoin_vm_types::block_metadata::BlockMetadata;
use starcoin_vm_types::transaction::{
    Module, Package, RawUserTransaction, Script, SignedUserTransaction, TransactionInfo,
};
use std::any::type_name;
use std::path::{Path, PathBuf};

/// This test ensure all base type serialize and hash is compatible with previous version.
#[stest::test]
fn check_types() {
    //Transaction
    check_data::<Script>().unwrap();
    check_data::<Module>().unwrap();

    check_data_and_hash::<Package>().unwrap();
    check_data_and_hash::<RawUserTransaction>().unwrap();
    check_data_and_hash::<SignedUserTransaction>().unwrap();
    check_data_and_hash::<BlockMetadata>().unwrap();
    check_data_and_hash::<TransactionInfo>().unwrap();

    //Block
    check_data_and_hash::<BlockHeader>().unwrap();
    check_data_and_hash::<Block>().unwrap();
    check_data_and_hash::<BlockInfo>().unwrap();

    //Network
    check_data::<ChainStatus>().unwrap();
    check_data::<CompactBlock>().unwrap();
    check_data::<TransactionsMessage>().unwrap();
    check_data::<CompactBlockMessage>().unwrap();
}

const DATA_DIR: &str = "data";

fn basic_path<T>() -> PathBuf {
    let name = type_name::<T>().split("::").last().unwrap();
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(DATA_DIR);
    path.push(name);
    path
}

fn data_file<T>() -> PathBuf {
    let mut path = basic_path::<T>();
    path.push("data");
    path
}

fn hash_file<T>() -> PathBuf {
    let mut path = basic_path::<T>();
    path.push("hash");
    path
}

fn read_hex_data<P: AsRef<Path>>(path: P) -> Result<Option<Vec<u8>>> {
    if path.as_ref().exists() {
        debug!("Read data from {:?}", path.as_ref());
        let c = std::fs::read_to_string(path)?;
        let data = hex::decode(c.as_str())?;
        Ok(Some(data))
    } else {
        Ok(None)
    }
}

fn write_hex_data<P: AsRef<Path>>(path: P, data: Vec<u8>) -> Result<()> {
    debug!("Write data to {:?}", path.as_ref());
    let dir = path.as_ref().parent().unwrap();
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, hex::encode(data))?;
    Ok(())
}

fn check_data_and_hash<T: Sample + Serialize + DeserializeOwned + PlainCryptoHash + PartialEq>(
) -> Result<()> {
    let t = check_data::<T>()?;
    check_hash(t)?;
    Ok(())
}

fn check_hash<T: PlainCryptoHash>(t: T) -> Result<()> {
    let type_name = type_name::<T>();
    let new_hash = t.crypto_hash();
    let path = hash_file::<T>();
    if let Some(data) = read_hex_data(path.as_path())? {
        let hash = HashValue::from_slice(data.as_slice())?;
        ensure!(
            hash == new_hash,
            "Check type {}'s crypto hash fail, expect:{}, got: {}",
            type_name,
            hash,
            new_hash
        );
    } else {
        write_hex_data(path, new_hash.to_vec())?;
    }
    Ok(())
}

fn check_data<T: Sample + Serialize + DeserializeOwned + PartialEq>() -> Result<T> {
    let type_name = type_name::<T>();
    ensure!(
        T::sample() == T::sample(),
        "Type {}'s sample return result is not stable."
    );
    let path = data_file::<T>();
    if let Some(data) = read_hex_data(path.as_path())? {
        let t = T::decode(data.as_slice())?;

        let new_data = t.encode()?;
        ensure!(
            data == new_data,
            "Check type {}'s serialize/deserialize fail, expect:{}, got: {}",
            type_name,
            hex::encode(data),
            hex::encode(new_data)
        );
        info!("Check {} ok", type_name);
        Ok(t)
    } else {
        let t = T::sample();
        write_hex_data(path, t.encode()?)?;
        Ok(t)
    }
}
