// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::{RpcClient, state_root_option::StateRootOption};
use anyhow::{format_err, Result};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::state_api_types::VmType;
use starcoin_rpc_api::types::state_api_types::VmType::MoveVm2;
use starcoin_state_api::{ChainStateReader, StateView, StateWithProof, StateWithTableItemProof};
use starcoin_state_tree::AccountStateSetIterator;
use starcoin_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_state::AccountState,
    block::BlockNumber,
    state_set::{AccountStateSet, ChainStateSet},
};
use starcoin_vm_types::{
    state_store::state_key::StateKey,
    state_store::table::{TableHandle, TableInfo},
};

pub struct RemoteStateReader<'a> {
    //TODO add cache.
    client: &'a RpcClient,
    state_root: HashValue,
}

impl<'a> RemoteStateReader<'a> {
    pub(crate) fn new(
        client: &'a RpcClient,
        state_root_opt: StateRootOption,
    ) -> Result<Self> {
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

        Ok(Self {
            client,
            state_root,
        })
    }
}

impl<'a> ChainStateReader for RemoteStateReader<'a> {
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof> {
        self.client
            .state_get_with_proof_by_root(
                access_path.clone(),
                self.state_root,
                Some(MoveVm2),
            )
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
            .state_get_with_table_item_proof_by_root(
                *handle,
                key.to_vec(),
                self.state_root,
                self.vm_type.clone(),
            )
            .map(Into::into)
    }
    fn get_table_info(&self, address: AccountAddress) -> Result<Option<TableInfo>> {
        self.client
            .state_get_table_info(address, Some(MoveVm2))
            .map(|v| v.map(Into::into))
    }
}

impl<'a> StateView for RemoteStateReader<'a> {
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<Vec<u8>>> {
        match state_key {
            StateKey::AccessPath(access_path) => Ok(self
                .client
                .state_get_with_proof_by_root(
                    access_path.clone(),
                    self.state_root(),
                    self.vm_type.clone(),
                )?
                .state
                .map(|v| v.0)),
            StateKey::TableItem(table_item) => Ok(self
                .client
                .state_get_with_table_item_proof_by_root(
                    table_item.handle,
                    table_item.key.clone(),
                    self.state_root(),
                    self.vm_type.clone(),
                )?
                .key_proof
                .0
                .map(|v| v.0)),
        }
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
