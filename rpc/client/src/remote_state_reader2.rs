// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::{RpcClient, StateRootOption};
use anyhow::{format_err, Result};
use starcoin_vm2_crypto::HashValue;
use starcoin_vm2_state_api::{ChainStateReader, StateWithProof, StateWithTableItemProof};
use starcoin_vm2_state_tree::AccountStateSetIterator;
use starcoin_vm2_types::{
    account_address::AccountAddress,
    account_state::AccountState,
    state_set::{AccountStateSet, ChainStateSet},
};
use starcoin_vm2_vm_types::state_store::{
    errors::StateviewError,
    state_key::inner::StateKeyInner,
    state_key::StateKey,
    state_storage_usage::StateStorageUsage,
    state_value::StateValue,
    table::{TableHandle, TableInfo},
    TStateView,
};

pub struct RemoteStateReader<'a> {
    //TODO add cache.
    client: &'a RpcClient,
    state_root: HashValue,
}

impl<'a> RemoteStateReader<'a> {
    pub(crate) fn new(client: &'a RpcClient, state_root_opt: StateRootOption) -> Result<Self> {
        let state_root = match state_root_opt {
            StateRootOption::Latest => client.state_get_state_root2()?,
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
    fn get_with_proof(&self, state_key: &StateKey) -> Result<StateWithProof> {
        self.client
            .state_get_with_proof_by_root2(state_key.clone(), self.state_root)
            .map(Into::into)
    }

    fn get_account_state(&self, _address: &AccountAddress) -> Result<AccountState> {
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
            .state_get_with_table_item_proof_by_root2(*handle, key.to_vec(), self.state_root)
            .map(Into::into)
    }
    fn get_table_info(&self, address: AccountAddress) -> Result<TableInfo> {
        self.client
            .state_get_table_info2(address)
            .map(TableInfo::from)
    }
}

impl<'a> TStateView for RemoteStateReader<'a> {
    type Key = StateKey;

    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<StateValue>, StateviewError> {
        match state_key.inner() {
            StateKeyInner::AccessPath(_access_path) => Ok(self
                .client
                .state_get_with_proof_by_root2(state_key.clone(), self.state_root())?
                .state
                .map(|v| v.0))
            .map(|v| v.map(StateValue::from)),
            StateKeyInner::TableItem { handle, key } => Ok(self
                .client
                .state_get_with_table_item_proof_by_root2(*handle, key.clone(), self.state_root())?
                .key_proof
                .0
                .map(|v| v.0)
                .map(StateValue::from)),
            StateKeyInner::Raw(_) => Err(format_err!("Can not get raw state value.").into()),
        }
    }

    fn get_usage(&self) -> starcoin_vm2_vm_types::state_store::Result<StateStorageUsage> {
        unimplemented!("RemoteStateReader get_usage not implemented")
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
