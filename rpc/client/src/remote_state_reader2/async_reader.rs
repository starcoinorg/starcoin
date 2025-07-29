// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::async_client::AsyncRpcClient;
use crate::StateRootOption;
use anyhow::format_err;
use serde::de::DeserializeOwned;
use starcoin_crypto::HashValue;
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_vm_types::account_config::{AccountResource, BalanceResource};
use starcoin_vm2_vm_types::move_resource::MoveResource;
use starcoin_vm2_vm_types::state_store::state_key::StateKey;

pub struct AsyncRemoteStateReader<'a> {
    //TODO add cache.
    client: &'a AsyncRpcClient,
    state_root: HashValue,
}

impl<'a> AsyncRemoteStateReader<'a> {
    pub async fn create(
        client: &'a AsyncRpcClient,
        state_root_opt: StateRootOption,
    ) -> anyhow::Result<Self> {
        let state_root = match state_root_opt {
            StateRootOption::Latest => client.state_get_state_root().await?,
            StateRootOption::BlockHash(block_hash) => {
                let multi_state = client
                    .chain_get_vm_multi_state(block_hash)
                    .await?
                    .ok_or_else(|| {
                        format_err!("Can not find vm multi_state by hash:{}", block_hash)
                    })?;
                multi_state.state_root2
            }
            StateRootOption::BlockNumber(block_number) => {
                let block = client
                    .chain_get_block_by_number(block_number, None)
                    .await?
                    .ok_or_else(|| format_err!("Can not find block by number: {}", block_number))?;
                let block_hash = block.header.block_hash;
                let multi_state = client
                    .chain_get_vm_multi_state(block_hash)
                    .await?
                    .ok_or_else(|| {
                        format_err!("Can not find vm multi_state by hash:{}", block_hash)
                    })?;
                multi_state.state_root2
            }
        };
        Ok(Self { client, state_root })
    }

    async fn get_resource<R>(&self, address: AccountAddress) -> anyhow::Result<Option<R>>
    where
        R: MoveResource + DeserializeOwned,
    {
        let state_key = StateKey::resource_typed::<R>(&address)?;
        self.client
            .state_get_with_proof_by_root(state_key, self.state_root)
            .await?
            .state
            .map_or(Ok(None), |state| {
                Some(bcs_ext::from_bytes::<R>(state.0.as_slice())).transpose()
            })
    }

    pub async fn get_balance(&self, address: AccountAddress) -> anyhow::Result<Option<u128>> {
        self.get_resource::<BalanceResource>(address)
            .await
            .map(|r| r.map(|resource| resource.token()))
    }

    pub async fn get_account_resource(
        &self,
        address: &AccountAddress,
    ) -> anyhow::Result<Option<AccountResource>> {
        self.get_resource::<AccountResource>(*address).await
    }
}
