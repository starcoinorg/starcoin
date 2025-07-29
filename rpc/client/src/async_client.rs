// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::{map_err, RpcClient};
use futures::{TryStream, TryStreamExt};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::GetBlockOption;
use starcoin_rpc_api::node::NodeInfo;
use starcoin_rpc_api::types::{BlockView, MintedBlockView, MultiStateView};
use starcoin_types::system_events::MintBlockEvent;
use starcoin_vm2_account_api::AccountInfo;
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_types::view::StateWithProofView;
use starcoin_vm2_vm_types::state_store::state_key::StateKey;
use starcoin_vm2_vm_types::transaction::{RawUserTransaction, SignedUserTransaction};

impl RpcClient {
    pub async fn account_default_async(&self) -> anyhow::Result<Option<AccountInfo>> {
        self.call_rpc_async(|inner| inner.account_client2.default())
            .await
            .map_err(map_err)
    }

    pub async fn account_get_async(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Option<AccountInfo>> {
        self.call_rpc_async(|inner| inner.account_client2.get(address))
            .await
            .map_err(map_err)
    }
    pub async fn account_unlock_async(
        &self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> anyhow::Result<AccountInfo> {
        self.call_rpc_async(|inner| {
            inner
                .account_client2
                .unlock(address, password, Some(duration.as_secs() as u32))
        })
        .await
        .map_err(map_err)
    }
    pub async fn account_sign_txn_async(
        &self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> anyhow::Result<SignedUserTransaction> {
        self.call_rpc_async(|inner| inner.account_client2.sign_txn(raw_txn, signer_address))
            .await
            .map_err(map_err)
    }
    pub async fn account_create_async(&self, password: String) -> anyhow::Result<AccountInfo> {
        self.call_rpc_async(|inner| inner.account_client2.create(password))
            .await
            .map_err(map_err)
    }
    pub async fn submit_txn_async(
        &self,
        signed_txn: SignedUserTransaction,
    ) -> anyhow::Result<HashValue> {
        self.call_rpc_async(|inner| inner.txpool_client.submit_transaction2(signed_txn))
            .await
            .map_err(map_err)
    }
    pub async fn state_get_with_proof_by_root_async(
        &self,
        state_key: StateKey,
        state_root: HashValue,
    ) -> anyhow::Result<StateWithProofView> {
        self.call_rpc_async(|inner| {
            inner
                .state_client2
                .get_with_proof_by_root(state_key, state_root)
        })
        .await
        .map_err(map_err)
    }
    pub async fn state_get_state_root_async(&self) -> anyhow::Result<HashValue> {
        self.call_rpc_async(|inner| inner.state_client2.get_state_root())
            .await
            .map_err(map_err)
    }
    pub async fn chain_get_block_by_number_async(
        &self,
        number: u64,
        opt: Option<GetBlockOption>,
    ) -> anyhow::Result<Option<BlockView>> {
        self.call_rpc_async(|inner| inner.chain_client.get_block_by_number(number, opt))
            .await
            .map_err(map_err)
    }
    pub async fn chain_get_vm_multi_state_async(
        &self,
        hash_value: HashValue,
    ) -> anyhow::Result<Option<MultiStateView>> {
        self.call_rpc_async(|inner| inner.chain_client.get_vm_multi_state(hash_value))
            .await
            .map_err(map_err)
    }
    pub async fn node_info_async(&self) -> anyhow::Result<NodeInfo> {
        self.call_rpc_async(|inner| inner.node_client.info())
            .await
            .map_err(map_err)
    }

    pub async fn miner_submit_async(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> anyhow::Result<MintedBlockView> {
        self.call_rpc_async(|inner| inner.miner_client.submit(minting_blob, nonce, extra))
            .await
            .map_err(map_err)
    }

    pub async fn subscribe_new_mint_blocks_async(
        &self,
    ) -> anyhow::Result<impl TryStream<Ok = MintBlockEvent, Error = anyhow::Error>> {
        self.call_rpc_async(|inner| async move {
            let res = inner.pubsub_client.subscribe_new_mint_block().await;
            res.map(|s| s.map_err(map_err))
        })
        .await
        .map_err(map_err)
    }
}
