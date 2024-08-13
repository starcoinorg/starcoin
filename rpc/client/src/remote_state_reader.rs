// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::RpcClient;
use anyhow::{format_err, Result};
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainStateReader, StateView, StateWithProof, StateWithTableItemProof};
use starcoin_state_tree::AccountStateSetIterator;
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::block::BlockNumber;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet};
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::state_value::StateValue;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use starcoin_vm_types::state_view::TStateView;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Default)]
pub enum StateRootOption {
    #[default]
    Latest,
    BlockHash(HashValue),
    BlockNumber(BlockNumber),
}

impl FromStr for StateRootOption {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(number) = s.parse::<u64>() {
            Ok(Self::BlockNumber(number))
        } else {
            Ok(Self::BlockHash(HashValue::from_str(s)?))
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

    fn get_account_state_set(&self, _address: &AccountAddress) -> Result<Option<AccountStateSet>> {
        unimplemented!()
    }

    fn state_root(&self) -> HashValue {
        //TODO change trait api to return Result<HashValue>
        self.state_root
    }
    fn dump(&self) -> Result<ChainStateSet> {
        unimplemented!()
    }

    fn dump_iter(&self) -> Result<AccountStateSetIterator> {
        unimplemented!()
    }

    fn get_with_table_item_proof(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> Result<StateWithTableItemProof> {
        self.client
            .state_get_with_table_item_proof_by_root(*handle, key.to_vec(), self.state_root)
            .map(Into::into)
    }
    fn get_table_info(&self, address: AccountAddress) -> Result<Option<TableInfo>> {
        self.client
            .state_get_table_info(address)
            .map(|v| v.map(Into::into))
    }
}

impl<'a> TStateView for RemoteStateReader<'a> {
    type Key = StateKey;

    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<StateValue>> {
        match state_key {
            StateKey::AccessPath(access_path) => Ok(self
                .client
                .state_get_with_proof_by_root(access_path.clone(), self.state_root())?
                .state
                .map(|v| v.0))
            .map(|v| v.map(|v| StateValue::from(v))),
            StateKey::TableItem(table_item) => Ok(self
                .client
                .state_get_with_table_item_proof_by_root(
                    table_item.handle,
                    table_item.key.clone(),
                    self.state_root(),
                )?
                .key_proof
                .0
                .map(|v| v.0)
                .map(|v| StateValue::from(v))),
        }
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
