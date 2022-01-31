// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use bcs_ext::Sample;
use network_api::messages::{CompactBlockMessage, TransactionsMessage};
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
    Module, Package, RawUserTransaction, Script, ScriptFunction, SignedUserTransaction,
    TransactionInfo,
};
use std::any::type_name;
use std::path::PathBuf;

/// This test ensure all base type serialize and hash is compatible with previous version.
#[stest::test]
fn check_types() {
    //Transaction
    check_data::<Script>().unwrap();
    check_data::<Module>().unwrap();
    check_data::<ScriptFunction>().unwrap();

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

fn json_file<T>() -> PathBuf {
    let mut path = basic_path::<T>();
    path.push("json");
    path
}

fn read_and_check_data<T: Serialize + DeserializeOwned + PartialEq>() -> Result<Option<T>> {
    let data_path = data_file::<T>();
    let json_path = json_file::<T>();
    if data_path.exists() && json_path.exists() {
        debug!("Read data from {:?}", data_path);
        let data = hex::decode(std::fs::read_to_string(data_path)?.as_str())?;
        let data_t = bcs_ext::from_bytes::<T>(data.as_slice())?;
        let json_t = serde_json::from_str::<T>(std::fs::read_to_string(json_path)?.as_str())?;
        ensure!(
            data_t == json_t,
            "{}'s bcs and json serialize data is not equals.",
            type_name::<T>()
        );

        let new_data = bcs_ext::to_bytes(&data_t)?;
        ensure!(
            data == new_data,
            "Check type {}'s serialize/deserialize fail, expect:{}, got: {}",
            type_name::<T>(),
            hex::encode(data),
            hex::encode(new_data)
        );
        Ok(Some(data_t))
    } else {
        Ok(None)
    }
}

fn write_data<T: Serialize>(t: &T) -> Result<()> {
    let data_path = data_file::<T>();
    let json_path = json_file::<T>();
    debug!("Write data to {:?}", data_path);
    let dir = data_path.parent().unwrap();
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(data_path, hex::encode(bcs_ext::to_bytes(t)?))?;
    std::fs::write(json_path, serde_json::to_string_pretty(t)?)?;
    Ok(())
}

fn read_hash<T>() -> Result<Option<HashValue>> {
    let path = hash_file::<T>();
    if path.exists() {
        debug!("Read hash from {:?}", path);
        let data = HashValue::from_slice(
            hex::decode(std::fs::read_to_string(path)?.as_str())?.as_slice(),
        )?;
        Ok(Some(data))
    } else {
        Ok(None)
    }
}

fn write_hash<T>(hash: HashValue) -> Result<()> {
    let path = hash_file::<T>();
    debug!("Write hash to {:?}", path);
    let dir = path.parent().unwrap();
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, hex::encode(hash.to_vec()))?;
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
    if let Some(hash) = read_hash::<T>()? {
        ensure!(
            hash == new_hash,
            "Check type {}'s crypto hash fail, expect:{}, got: {}",
            type_name,
            hash,
            new_hash
        );
    } else {
        write_hash::<T>(new_hash)?;
    }
    Ok(())
}

fn check_data<T: Sample + Serialize + DeserializeOwned + PartialEq>() -> Result<T> {
    let type_name = type_name::<T>();
    ensure!(
        T::sample() == T::sample(),
        "Type {}'s sample return result is not stable.",
        type_name
    );
    if let Some(t) = read_and_check_data::<T>()? {
        info!("Check {} ok", type_name);
        Ok(t)
    } else {
        let t = T::sample();
        write_data(&t)?;
        Ok(t)
    }
}
