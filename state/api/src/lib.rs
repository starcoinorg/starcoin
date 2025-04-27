// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::{StateRequest, StateResponse};
use anyhow::{format_err, Result};
use once_cell::sync::Lazy;
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, ServiceHandler, ServiceRef};
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};
use std::str::FromStr;

pub use chain_state::{
    AccountStateReader, ChainStateReader, ChainStateWriter, StateProof, StateWithProof,
    StateWithTableItemProof,
};
pub use starcoin_state_tree::StateNodeStore;
use starcoin_types::state_set::AccountStateSet;
use starcoin_vm_types::access_path::DataPath;
use starcoin_vm_types::account_config::TABLE_HANDLE_ADDRESS_LIST;
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
pub use starcoin_vm_types::state_view::StateReaderExt;

mod chain_state;
pub mod message;
pub mod mock;
use bytes::Bytes;

pub static TABLE_PATH_LIST: Lazy<Vec<DataPath>> = Lazy::new(|| {
    let mut path_list = vec![];
    for handle_address in &*TABLE_HANDLE_ADDRESS_LIST {
        let str = format!(
            "{}/1/{}::TableHandles::TableHandles",
            handle_address, handle_address,
        );
        path_list.push(AccessPath::from_str(str.as_str()).unwrap().path);
    }
    path_list
});

#[async_trait::async_trait]
pub trait ChainStateAsyncService: Clone + std::marker::Unpin + Send + Sync {
    async fn get(self, state_key: StateKey) -> Result<Option<Bytes>>;

    async fn get_with_proof(self, state_key: StateKey) -> Result<StateWithProof>;

    async fn get_resource<R>(self, address: AccountAddress) -> Result<R>
    where
        R: MoveResource,
    {
        let rsrc_bytes = self
            .get(StateKey::resource_typed::<R>(&address)?)
            .await?
            .ok_or_else(|| {
                format_err!(
                    "Resource {:?} not exists at address:{}",
                    R::module_identifier(),
                    address
                )
            })?;
        let rsrc = bcs_ext::from_bytes::<R>(&rsrc_bytes)?;
        Ok(rsrc)
    }

    async fn get_account_state(self, address: AccountAddress) -> Result<AccountState>;

    /// get account stateset on state_root(if empty, use current state root).
    async fn get_account_state_set(
        self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> Result<AccountStateSet>;

    async fn state_root(self) -> Result<HashValue>;

    async fn get_with_proof_by_root(
        self,
        state_key: StateKey,
        state_root: HashValue,
    ) -> Result<StateWithProof>;

    async fn get_account_state_by_root(
        self,
        address: AccountAddress,
        state_root: HashValue,
    ) -> Result<AccountState>;

    async fn get_with_table_item_proof(
        self,
        handle: TableHandle,
        key: Vec<u8>,
    ) -> Result<StateWithTableItemProof>;
    async fn get_with_table_item_proof_by_root(
        self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> Result<StateWithTableItemProof>;

    async fn get_table_info(self, address: AccountAddress) -> Result<TableInfo>;
}

#[async_trait::async_trait]
impl<S> ChainStateAsyncService for ServiceRef<S>
where
    S: ActorService + ServiceHandler<S, StateRequest>,
{
    async fn get(self, state_key: StateKey) -> Result<Option<Bytes>> {
        let response = self.send(StateRequest::Get(state_key)).await??;
        if let StateResponse::State(state) = response {
            Ok(state)
        } else {
            panic!("Unexpected response type.")
    }

    async fn get_with_proof(self, state_key: StateKey) -> Result<StateWithProof> {
        let response = self.send(StateRequest::GetWithProof(state_key)).await??;
        if let StateResponse::StateWithProof(state) = response {
            Ok(*state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_account_state(self, address: AccountAddress) -> Result<AccountState> {
        let response = self.send(StateRequest::GetAccountState(address)).await??;
        if let StateResponse::AccountState(state) = response {
            Ok(state
                .ok_or_else(|| format_err!("AccountState not exists for address: {}", address))?)
        } else {
            panic!("Unexpect response type.")
        }
    }
    async fn get_account_state_set(
        self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> Result<AccountStateSet> {
        let response = self
            .send(StateRequest::GetAccountStateSet {
                address,
                state_root,
            })
            .await??;
        if let StateResponse::AccountStateSet(state) = response {
            Ok(state.ok_or_else(|| {
                format_err!("AccountStateSet not exists for address: {}", address)
            })?)
        } else {
            panic!("Unexpected response type.")
        }
    }
    async fn state_root(self) -> Result<HashValue> {
        let response = self.send(StateRequest::StateRoot()).await??;
        if let StateResponse::StateRoot(root) = response {
            Ok(root)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_with_proof_by_root(
        self,
        state_key: StateKey,
        state_root: HashValue,
    ) -> Result<StateWithProof> {
        let response = self
            .send(StateRequest::GetWithProofByRoot(
                state_key.clone(),
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
    ) -> Result<AccountState> {
        let response = self
            .send(StateRequest::GetAccountStateByRoot(
                account_address,
                state_root,
            ))
            .await??;
        if let StateResponse::AccountState(state) = response {
            Ok(state.ok_or_else(|| {
                format_err!("AccountState not exists for address: {}", account_address)
            })?)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_with_table_item_proof(
        self,
        handle: TableHandle,
        key: Vec<u8>,
    ) -> Result<StateWithTableItemProof> {
        let response = self
            .send(StateRequest::GetWithTableItemProof(handle, key))
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
    ) -> Result<StateWithTableItemProof> {
        let response = self
            .send(StateRequest::GetWithTableItemProofByRoot(
                handle, key, state_root,
            ))
            .await??;
        if let StateResponse::StateWithTableItemProof(state) = response {
            Ok(*state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_table_info(self, address: AccountAddress) -> Result<TableInfo> {
        let response = self.send(StateRequest::GetTableInfo(address)).await??;
        if let StateResponse::TableInfo(state) = response {
            Ok(
                state
                    .ok_or_else(|| format_err!("TableInfo not exists for address: {}", address))?,
            )
        } else {
            panic!("Unexpect response type.")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::TABLE_PATH_LIST;

    #[test]
    fn test_table_path_list() {
        let mut path_list = vec![];
        let str_list = vec![
            "1/0x00000000000000000000000000000031::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000032::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000033::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000034::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000035::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000036::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000037::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000038::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000039::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000003a::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000003b::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000003c::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000003d::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000003e::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000003f::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000040::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000041::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000042::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000043::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000044::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000045::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000046::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000047::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000048::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000049::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000004a::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000004b::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000004c::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000004d::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000004e::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000004f::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000050::TableHandles::TableHandles",
        ];
        for table_path in TABLE_PATH_LIST.iter() {
            path_list.push(format!("{}", table_path));
        }
        assert_eq!(path_list, str_list);
    }
}
