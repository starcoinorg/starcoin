// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::RpcClient;
use anyhow::{format_err, Result};
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainStateReader, StateView, StateWithProof};
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::block::BlockNumber;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet};
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum StateRootOption {
    Latest,
    BlockHash(HashValue),
    BlockNumber(BlockNumber),
}

impl Default for StateRootOption {
    fn default() -> Self {
        StateRootOption::Latest
    }
}

impl FromStr for StateRootOption {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(number) = s.parse::<u64>() {
            Ok(StateRootOption::BlockNumber(number))
        } else {
            Ok(StateRootOption::BlockHash(HashValue::from_str(s)?))
        }
    }
}

pub struct RemoteStateReader<'a> {
    //TODO add cache.
    client: &'a RpcClient,
    state_root: HashValue,
}

impl<'a> RemoteStateReader<'a> {
    pub(crate) fn new(client: &'a RpcClient, state_root_opt: StateRootOption) -> Result<Self> {
        let state_root = match state_root_opt {
            StateRootOption::Latest => client.state_get_state_root()?,
            StateRootOption::BlockHash(block_hash) => {
                let block = client
                    .chain_get_block_by_hash(block_hash, None)?
                    .ok_or_else(|| format_err!("Can not find block by hash:{}", block_hash))?;
                block.header.state_root
            }
            StateRootOption::BlockNumber(block_number) => {
                let block = client
                    .chain_get_block_by_number(block_number, None)?
                    .ok_or_else(|| format_err!("Can not find block by number: {}", block_number))?;
                block.header.state_root
            }
        };

        Ok(Self::new_with_root(client, state_root))
    }

    fn new_with_root(client: &'a RpcClient, state_root: HashValue) -> Self {
        Self { client, state_root }
    }
}

impl<'a> ChainStateReader for RemoteStateReader<'a> {
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof> {
        self.client
            .state_get_with_proof_by_root(access_path.clone(), self.state_root)
            .map(Into::into)
    }

    fn get_account_state(&self, _address: &AccountAddress) -> Result<Option<AccountState>> {
        unimplemented!()
        //TODO implement get_account_state by root
    }

    fn state_root(&self) -> HashValue {
        //TODO change trait api to return Result<HashValue>
        self.state_root
    }

    fn get_account_state_set(&self, _address: &AccountAddress) -> Result<Option<AccountStateSet>> {
        unimplemented!()
    }
    fn dump(&self) -> Result<ChainStateSet> {
        unimplemented!()
    }
}

impl<'a> StateView for RemoteStateReader<'a> {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        Ok(self
            .client
            .state_get_with_proof_by_root(access_path.clone(), self.state_root())?
            .state
            .map(|v| v.0))
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
