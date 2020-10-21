// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{common::strip, config::global::Config as GlobalConfig, errors::*};
use starcoin_crypto::HashValue;
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_vm_types::genesis_config::ChainId;
use std::str::FromStr;

#[derive(Debug)]
pub enum Entry {
    Author(String),
    Timestamp(u64),
    Number(u64),
    Uncles(u64),
}

impl FromStr for Entry {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.split_whitespace().collect::<String>();
        let s = strip(&s, "//!")
            .ok_or_else(|| ErrorKind::Other("txn config entry must start with //!".to_string()))?
            .trim_start();

        if let Some(s) = strip(s, "author:") {
            if s.is_empty() {
                return Err(ErrorKind::Other("sender cannot be empty".to_string()).into());
            }
            return Ok(Entry::Author(s.to_string()));
        }

        if let Some(s) = strip(s, "block-time:") {
            return Ok(Entry::Timestamp(s.parse::<u64>()?));
        }
        if let Some(s) = strip(s, "block-number:") {
            return Ok(Entry::Number(s.parse::<u64>()?));
        }
        if let Some(s) = strip(s, "block-uncles:") {
            return Ok(Entry::Uncles(s.parse::<u64>()?));
        }
        Err(ErrorKind::Other(format!(
            "failed to parse '{}' as transaction config entry",
            s
        ))
        .into())
    }
}

/// Checks whether a line denotes the start of a new transaction.
pub fn is_new_block(s: &str) -> bool {
    let s = s.trim();
    if !s.starts_with("//!") {
        return false;
    }
    s[3..].trim_start() == "block-prologue"
}

impl Entry {
    pub fn try_parse(s: &str) -> Result<Option<Self>> {
        if s.starts_with("//!") {
            Ok(Some(s.parse::<Entry>()?))
        } else {
            Ok(None)
        }
    }
}

pub fn build_block_metadata(config: &GlobalConfig, entries: &[Entry]) -> Result<BlockMetadata> {
    let mut timestamp = None;
    let mut author = None;
    let mut author_public_key = None;
    let mut number = None;
    let mut uncles = 0u64;

    for entry in entries {
        match entry {
            Entry::Author(s) => {
                let account = config.get_account_for_name(s)?;
                author = Some(*account.address());
                author_public_key = Some(account.clone().pubkey);
            }
            Entry::Timestamp(new_timestamp) => timestamp = Some(new_timestamp),
            Entry::Number(new_number) => number = Some(new_number),
            Entry::Uncles(new_uncles) => uncles = *new_uncles,
        }
    }
    //TODO support read timestamp from FakeExecutor's net.time_service()
    if let (Some(t), Some(author), Some(number)) = (timestamp, author, number) {
        Ok(BlockMetadata::new(
            HashValue::random(),
            *t,
            author,
            author_public_key,
            uncles,
            *number,
            ChainId::test(),
            0,
        ))
    } else {
        Err(ErrorKind::Other("Cannot generate block metadata".to_string()).into())
    }
}
