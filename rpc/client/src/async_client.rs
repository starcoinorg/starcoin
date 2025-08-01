// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::{map_err, ConnSource, RpcClientInner};
use futures::{TryStream, TryStreamExt};
use jsonrpc_client_transports::transports::{ipc, ws};
use jsonrpc_client_transports::{RpcChannel, RpcError};
use log::{error, info};
use parking_lot::Mutex;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::GetBlockOption;
use starcoin_rpc_api::node::NodeInfo;
use starcoin_rpc_api::types::{BlockView, MintedBlockView, MultiStateView};
use starcoin_types::system_events::MintBlockEvent;
use starcoin_vm2_account_api::AccountInfo;
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_types::view::{StateWithProofView, TransactionInfoView};
use starcoin_vm2_vm_types::state_store::state_key::StateKey;
use starcoin_vm2_vm_types::transaction::{RawUserTransaction, SignedUserTransaction};

pub struct AsyncRpcClient {
    inner: Mutex<Option<RpcClientInner>>,
    provider: AsyncConnProvider,
}

struct AsyncConnProvider {
    conn_source: ConnSource,
}

impl AsyncConnProvider {
    pub fn new(conn_source: ConnSource) -> Self {
        Self { conn_source }
    }
    pub async fn get_rpc_channel_async(&self) -> anyhow::Result<RpcChannel, RpcError> {
        match self.conn_source.clone() {
            ConnSource::Ipc(sock_path) => ipc::connect(sock_path).await,
            ConnSource::WebSocket(url) => ws::try_connect(url.as_str())?.await,
            ConnSource::Local(channel) => Ok(*channel),
        }
    }
}

impl AsyncRpcClient {
    pub async fn new(conn_source: ConnSource) -> anyhow::Result<Self> {
        let provider = AsyncConnProvider::new(conn_source);
        let inner: RpcClientInner = provider
            .get_rpc_channel_async()
            .await
            .map_err(map_err)?
            .into();

        Ok(Self {
            inner: Mutex::new(Some(inner)),
            provider,
        })
    }
    async fn call_rpc_async<F, T>(
        &self,
        f: impl FnOnce(RpcClientInner) -> F + Send,
    ) -> Result<T, RpcError>
    where
        F: std::future::Future<Output = Result<T, RpcError>> + Send,
    {
        let inner_opt = self.inner.lock().as_ref().cloned();
        let inner = match inner_opt {
            Some(inner) => inner,
            None => {
                info!(
                    "Connection is lost, try reconnect by {:?}",
                    &self.provider.conn_source
                );
                let new_inner: RpcClientInner = self
                    .provider
                    .get_rpc_channel_async()
                    .await
                    .map(|c| c.into())?;
                *(self.inner.lock()) = Some(new_inner.clone());
                new_inner
            }
        };
        let result = f(inner).await;
        if let Err(RpcError::Other(e)) = &result {
            error!("rpc error due to {}", e);
            *(self.inner.lock()) = None;
        }
        result
    }
    pub async fn account_default(&self) -> anyhow::Result<Option<AccountInfo>> {
        self.call_rpc_async(|inner| inner.account_client2.default())
            .await
            .map_err(map_err)
    }

    pub async fn account_get(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Option<AccountInfo>> {
        self.call_rpc_async(|inner| inner.account_client2.get(address))
            .await
            .map_err(map_err)
    }
    pub async fn account_unlock(
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
    pub async fn account_sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> anyhow::Result<SignedUserTransaction> {
        self.call_rpc_async(|inner| inner.account_client2.sign_txn(raw_txn, signer_address))
            .await
            .map_err(map_err)
    }
    pub async fn account_create(&self, password: String) -> anyhow::Result<AccountInfo> {
        self.call_rpc_async(|inner| inner.account_client2.create(password))
            .await
            .map_err(map_err)
    }
    pub async fn submit_txn(&self, signed_txn: SignedUserTransaction) -> anyhow::Result<HashValue> {
        self.call_rpc_async(|inner| inner.txpool_client.submit_transaction2(signed_txn))
            .await
            .map_err(map_err)
    }
    pub async fn submit_hex_txn(&self, txn: String) -> anyhow::Result<HashValue> {
        self.call_rpc_async(|inner| inner.txpool_client.submit_hex_transaction2(txn))
            .await
            .map_err(map_err)
    }
    pub async fn next_sequence_number_in_txpool(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Option<u64>> {
        self.call_rpc_async(|inner| inner.txpool_client.next_sequence_number2(address))
            .await
            .map_err(map_err)
    }
    pub async fn state_get_with_proof_by_root(
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
    pub async fn state_get_state_root(&self) -> anyhow::Result<HashValue> {
        self.call_rpc_async(|inner| inner.state_client2.get_state_root())
            .await
            .map_err(map_err)
    }
    pub async fn chain_get_transaction_info(
        &self,
        hash: HashValue,
    ) -> anyhow::Result<Option<TransactionInfoView>> {
        self.call_rpc_async(|inner| inner.chain_client.get_transaction_info2(hash))
            .await
            .map_err(map_err)
    }
    pub async fn chain_get_block_by_number(
        &self,
        number: u64,
        opt: Option<GetBlockOption>,
    ) -> anyhow::Result<Option<BlockView>> {
        self.call_rpc_async(|inner| inner.chain_client.get_block_by_number(number, opt))
            .await
            .map_err(map_err)
    }
    pub async fn chain_get_vm_multi_state(
        &self,
        hash_value: HashValue,
    ) -> anyhow::Result<Option<MultiStateView>> {
        self.call_rpc_async(|inner| inner.chain_client.get_vm_multi_state(hash_value))
            .await
            .map_err(map_err)
    }
    pub async fn node_info(&self) -> anyhow::Result<NodeInfo> {
        self.call_rpc_async(|inner| inner.node_client.info())
            .await
            .map_err(map_err)
    }

    pub async fn miner_submit(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> anyhow::Result<MintedBlockView> {
        self.call_rpc_async(|inner| inner.miner_client.submit(minting_blob, nonce, extra))
            .await
            .map_err(map_err)
    }

    pub async fn subscribe_new_mint_blocks(
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
