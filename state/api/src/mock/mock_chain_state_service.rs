// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ChainStateAsyncService, StateWithProof, StateWithTableItemProof};
use anyhow::Result;
use bytes::Bytes;
use starcoin_crypto::HashValue;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::state_set::AccountStateSet;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};

//TODO implement Mock service
#[derive(Clone, Default)]
pub struct MockChainStateService {}

impl MockChainStateService {
    pub fn new() -> Self {
        Self::default()
    }
}

#[allow(clippy::diverging_sub_expression)]
#[async_trait::async_trait]
impl ChainStateAsyncService for MockChainStateService {
    async fn get(self, _state_key: StateKey) -> Result<Option<Bytes>> {
        unimplemented!()
    }

    async fn get_with_proof(self, _state_key: StateKey) -> Result<StateWithProof> {
        unimplemented!()
    }

    async fn get_account_state(self, _address: AccountAddress) -> Result<AccountState> {
        unimplemented!()
    }

    async fn get_account_state_set(
        self,
        _address: AccountAddress,
        _state_root: Option<HashValue>,
    ) -> Result<AccountStateSet> {
        unimplemented!()
    }

    async fn state_root(self) -> Result<HashValue> {
        unimplemented!()
    }

    async fn get_with_proof_by_root(
        self,
        _state_key: StateKey,
        _state_root: HashValue,
    ) -> Result<StateWithProof> {
        unimplemented!()
    }

    async fn get_account_state_by_root(
        self,
        _address: AccountAddress,
        _state_root: HashValue,
    ) -> Result<AccountState> {
        unimplemented!()
    }

    async fn get_with_table_item_proof(
        self,
        _handle: TableHandle,
        _key: Vec<u8>,
    ) -> Result<StateWithTableItemProof> {
        unimplemented!()
    }

    async fn get_with_table_item_proof_by_root(
        self,
        _handle: TableHandle,
        _key: Vec<u8>,
        _state_root: HashValue,
    ) -> Result<StateWithTableItemProof> {
        unimplemented!()
    }

    async fn get_table_info(self, _address: AccountAddress) -> Result<TableInfo> {
        unimplemented!()
    }
}
