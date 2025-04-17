// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chain_state_async_service::ChainStateAsyncService,
    message::{StateRequest, StateRequestVMType::MoveVm1, StateResponse},
    StateWithProof, StateWithTableItemProof,
};
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, ServiceHandler, ServiceRef};
use starcoin_types::{
    account_address::AccountAddress, account_state::AccountState, state_set::AccountStateSet,
};
use starcoin_vm_types::{
    access_path::AccessPath,
    state_store::table::{TableHandle, TableInfo},
};

#[async_trait::async_trait]
impl<S> ChainStateAsyncService for ServiceRef<S>
where
    S: ActorService + ServiceHandler<S, StateRequest>,
{
    async fn get(self, access_path: AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        let response = self
            .send(StateRequest::Get(MoveVm1, access_path))
            .await??;
        if let StateResponse::State(state) = response {
            Ok(state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_with_proof(self, access_path: AccessPath) -> anyhow::Result<StateWithProof> {
        let response = self
            .send(StateRequest::GetWithProof(MoveVm1, access_path))
            .await??;
        if let StateResponse::StateWithProof(state) = response {
            Ok(*state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_account_state(
        self,
        address: AccountAddress,
    ) -> anyhow::Result<Option<AccountState>> {
        let response = self
            .send(StateRequest::GetAccountState(MoveVm1, address))
            .await??;
        if let StateResponse::AccountState(state) = response {
            Ok(state)
        } else {
            panic!("Unexpect response type.")
        }
    }
    async fn get_account_state_set(
        self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> anyhow::Result<Option<AccountStateSet>> {
        let response = self
            .send(StateRequest::GetAccountStateSet {
                vm_type: MoveVm1,
                address,
                state_root,
            })
            .await??;
        if let StateResponse::AccountStateSet(state) = response {
            Ok(state)
        } else {
            panic!("Unexpected response type.")
        }
    }
    async fn state_root(self) -> anyhow::Result<HashValue> {
        let response = self.send(StateRequest::StateRoot(MoveVm1)).await??;
        if let StateResponse::StateRoot(root) = response {
            Ok(root)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_with_proof_by_root(
        self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> anyhow::Result<StateWithProof> {
        let response = self
            .send(StateRequest::GetWithProofByRoot(
                MoveVm1,
                access_path,
                state_root,
            ))
            .await??;
        if let StateResponse::StateWithProof(state) = response {
            Ok(*state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_account_state_by_root(
        self,
        account_address: AccountAddress,
        state_root: HashValue,
    ) -> anyhow::Result<Option<AccountState>> {
        let response = self
            .send(StateRequest::GetAccountStateByRoot(
                MoveVm1,
                account_address,
                state_root,
            ))
            .await??;
        if let StateResponse::AccountState(state) = response {
            Ok(state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_with_table_item_proof(
        self,
        handle: TableHandle,
        key: Vec<u8>,
    ) -> anyhow::Result<StateWithTableItemProof> {
        let response = self
            .send(StateRequest::GetWithTableItemProof(MoveVm1, handle, key))
            .await??;
        if let StateResponse::StateWithTableItemProof(state) = response {
            Ok(*state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_with_table_item_proof_by_root(
        self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> anyhow::Result<StateWithTableItemProof> {
        let response = self
            .send(StateRequest::GetWithTableItemProofByRoot(
                MoveVm1, handle, key, state_root,
            ))
            .await??;
        if let StateResponse::StateWithTableItemProof(state) = response {
            Ok(*state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_table_info(self, address: AccountAddress) -> anyhow::Result<Option<TableInfo>> {
        let response = self
            .send(StateRequest::GetTableInfo(MoveVm1, address))
            .await??;
        if let StateResponse::TableInfo(state) = response {
            Ok(state)
        } else {
            panic!("Unexpect response type.")
        }
    }
}
