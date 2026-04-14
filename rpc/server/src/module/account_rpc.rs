// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::helpers::TransactionRequestFiller;
use jsonrpsee::core::{async_trait, RpcResult};
use starcoin_account_api::{AccountAsyncService, AccountInfo};

use starcoin_config::NodeConfig;
use starcoin_rpc_api::account::AccountApiServer;
use starcoin_rpc_api::types::{SignedMessageView, StrView, TransactionRequest};
use starcoin_state_api::ChainStateAsyncService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::sign_message::SigningMessage;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct AccountRpcImpl<Account, Pool, State>
where
    Account: AccountAsyncService + 'static,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
{
    account: Account,
    pool: Pool,
    chain_state: State,
    node_config: Arc<NodeConfig>,
}

impl<Account, Pool, State> AccountRpcImpl<Account, Pool, State>
where
    Account: AccountAsyncService,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
{
    pub fn new(
        node_config: Arc<NodeConfig>,
        account: Account,
        pool: Pool,
        chain_state: State,
    ) -> Self {
        Self {
            account,
            pool,
            chain_state,
            node_config,
        }
    }
    fn txn_request_filler(&self) -> TransactionRequestFiller<Account, Pool, State> {
        TransactionRequestFiller {
            account: Some(self.account.clone()),
            pool: self.pool.clone(),
            chain_state: self.chain_state.clone(),
            node_config: self.node_config.clone(),
        }
    }
}

#[async_trait]
impl<S, Pool, State> AccountApiServer for AccountRpcImpl<S, Pool, State>
where
    S: AccountAsyncService,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
{
    async fn default(&self) -> RpcResult<Option<AccountInfo>> {
        let service = self.account.clone();
        service
            .get_default_account()
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn set_default_account(&self, addr: AccountAddress) -> RpcResult<AccountInfo> {
        let service = self.account.clone();
        service
            .set_default_account(addr)
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn create(&self, password: String) -> RpcResult<AccountInfo> {
        let service = self.account.clone();
        service
            .create_account(password)
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn list(&self) -> RpcResult<Vec<AccountInfo>> {
        let service = self.account.clone();
        service
            .get_accounts()
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn get(&self, address: AccountAddress) -> RpcResult<Option<AccountInfo>> {
        let service = self.account.clone();
        service
            .get_account(address)
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn sign(
        &self,
        address: AccountAddress,
        data: SigningMessage,
    ) -> RpcResult<SignedMessageView> {
        let account_service = self.account.clone();
        let signed_message = account_service
            .sign_message(address, data)
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(signed_message.into())
    }

    async fn sign_txn_request(&self, txn_request: TransactionRequest) -> RpcResult<String> {
        let me = self.clone();
        let raw_txn = me
            .txn_request_filler()
            .fill_transaction(txn_request)
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        let sender = raw_txn.sender();
        let signed_txn = me
            .account
            .sign_txn(raw_txn, sender)
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        let signed_txn_bytes =
            bcs_ext::to_bytes(&signed_txn).map_err(crate::module::map_jsonrpc_err)?;
        Ok(format!("0x{}", hex::encode(signed_txn_bytes)))
    }

    async fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer: AccountAddress,
    ) -> RpcResult<SignedUserTransaction> {
        let service = self.account.clone();
        service
            .sign_txn(raw_txn, signer)
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn sign_txn_in_batch(
        &self,
        raw_txn: Vec<RawUserTransaction>,
    ) -> RpcResult<Vec<SignedUserTransaction>> {
        let service = self.account.clone();
        service
            .sign_txn_in_batch(raw_txn)
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: Option<u32>,
    ) -> RpcResult<AccountInfo> {
        let service = self.account.clone();
        service
            .unlock_account(
                address,
                password,
                Duration::from_secs(duration.unwrap_or(u32::MAX) as u64),
            )
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn unlock_in_batch(
        &self,
        batch: Vec<(AccountAddress, String)>,
        duration: Option<u32>,
    ) -> RpcResult<Vec<AccountInfo>> {
        let service = self.account.clone();
        service
            .unlock_account_in_batch(
                batch,
                Duration::from_secs(duration.unwrap_or(u32::MAX) as u64),
            )
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn lock(&self, address: AccountAddress) -> RpcResult<AccountInfo> {
        let service = self.account.clone();
        service
            .lock_account(address)
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    /// Import private key with address.
    async fn import(
        &self,
        address: AccountAddress,
        private_key: StrView<Vec<u8>>,
        password: String,
    ) -> RpcResult<AccountInfo> {
        let service = self.account.clone();
        service
            .import_account(address, private_key.0, password)
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn import_readonly(
        &self,
        address: AccountAddress,
        public_key: StrView<Vec<u8>>,
    ) -> RpcResult<AccountInfo> {
        let service = self.account.clone();
        service
            .import_readonly_account(address, public_key.0)
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    /// Return the private key as bytes for `address`
    async fn export(&self, address: AccountAddress, password: String) -> RpcResult<Vec<u8>> {
        let service = self.account.clone();
        service
            .export_account(address, password)
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn change_account_password(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> RpcResult<AccountInfo> {
        let account_service = self.account.clone();
        account_service
            .change_account_password(address, new_password)
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn accepted_tokens(&self, address: AccountAddress) -> RpcResult<Vec<TokenCode>> {
        let service = self.account.clone();
        service
            .accepted_tokens(address)
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn remove(
        &self,
        address: AccountAddress,
        password: Option<String>,
    ) -> RpcResult<AccountInfo> {
        let service = self.account.clone();
        service
            .remove_account(address, password)
            .await
            .map_err(crate::module::map_jsonrpc_err)
    }
}
