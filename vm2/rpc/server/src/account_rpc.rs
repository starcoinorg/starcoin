// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::helpers::TransactionRequestFiller as TransactionRequestFiller2;
use jsonrpsee::core::{async_trait, RpcResult};
use starcoin_vm2_account_api::{
    AccountAsyncService as AccountAsyncService2, AccountInfo as AccountInfo2,
};
use starcoin_vm2_state_api::ChainStateAsyncService as ChainStateAsyncService2;

use starcoin_config::NodeConfig;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_vm2_rpc_api::account_api::AccountApiServer as AccountApiServer2;
use starcoin_vm2_types::view::{
    SignedMessageView as SignedMessageView2, StrView as StrView2,
    TransactionRequest as TransactionRequest2,
};
use starcoin_vm2_types::{
    account_address::AccountAddress as AccountAddress2,
    account_config::token_code::TokenCode as TokenCode2,
    sign_message::SigningMessage as SigningMessage2,
    transaction::{
        RawUserTransaction as RawUserTransaction2, SignedUserTransaction as SignedUserTransaction2,
    },
};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct AccountRpcImpl<Account, Pool, State>
where
    Account: AccountAsyncService2 + 'static,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService2 + 'static,
{
    account: Account,
    pool: Pool,
    chain_state: State,
    node_config: Arc<NodeConfig>,
}

impl<Account, Pool, State> AccountRpcImpl<Account, Pool, State>
where
    Account: AccountAsyncService2,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService2 + 'static,
{
    #[allow(dead_code)]
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
    fn txn_request_filler(&self) -> TransactionRequestFiller2<Account, Pool, State> {
        TransactionRequestFiller2 {
            account: Some(self.account.clone()),
            pool: self.pool.clone(),
            chain_state: self.chain_state.clone(),
            node_config: self.node_config.clone(),
        }
    }
}

#[async_trait]
impl<S, Pool, State> AccountApiServer2 for AccountRpcImpl<S, Pool, State>
where
    S: AccountAsyncService2,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService2 + 'static,
{
    async fn default(&self) -> RpcResult<Option<AccountInfo2>> {
        let service = self.account.clone();
        service
            .get_default_account()
            .await
            .map_err(crate::map_jsonrpc_err)
    }

    async fn set_default_account(&self, addr: AccountAddress2) -> RpcResult<AccountInfo2> {
        let service = self.account.clone();
        service
            .set_default_account(addr)
            .await
            .map_err(crate::map_jsonrpc_err)
    }

    async fn create(&self, password: String) -> RpcResult<AccountInfo2> {
        let service = self.account.clone();
        service
            .create_account(password)
            .await
            .map_err(crate::map_jsonrpc_err)
    }

    async fn list(&self) -> RpcResult<Vec<AccountInfo2>> {
        let service = self.account.clone();
        service.get_accounts().await.map_err(crate::map_jsonrpc_err)
    }

    async fn get(&self, address: AccountAddress2) -> RpcResult<Option<AccountInfo2>> {
        let service = self.account.clone();
        service
            .get_account(address)
            .await
            .map_err(crate::map_jsonrpc_err)
    }

    async fn sign(
        &self,
        address: AccountAddress2,
        data: SigningMessage2,
    ) -> RpcResult<SignedMessageView2> {
        let account_service = self.account.clone();
        let signed_message = account_service
            .sign_message(address, data)
            .await
            .map_err(crate::map_jsonrpc_err)?;
        Ok(signed_message.into())
    }

    async fn sign_txn_request(&self, txn_request: TransactionRequest2) -> RpcResult<String> {
        let me = self.clone();
        let raw_txn = me
            .txn_request_filler()
            .fill_transaction(txn_request)
            .await
            .map_err(crate::map_jsonrpc_err)?;
        let sender = raw_txn.sender();
        let signed_txn = me
            .account
            .sign_txn(raw_txn, sender)
            .await
            .map_err(crate::map_jsonrpc_err)?;
        let signed_txn_bytes = bcs_ext::to_bytes(&signed_txn).map_err(crate::map_jsonrpc_err)?;
        Ok(format!("0x{}", hex::encode(signed_txn_bytes)))
    }

    async fn sign_txn(
        &self,
        raw_txn: RawUserTransaction2,
        signer: AccountAddress2,
    ) -> RpcResult<SignedUserTransaction2> {
        let service = self.account.clone();
        service
            .sign_txn(raw_txn, signer)
            .await
            .map_err(crate::map_jsonrpc_err)
    }

    async fn unlock(
        &self,
        address: AccountAddress2,
        password: String,
        duration: Option<u32>,
    ) -> RpcResult<AccountInfo2> {
        let service = self.account.clone();
        service
            .unlock_account(
                address,
                password,
                Duration::from_secs(duration.unwrap_or(u32::MAX) as u64),
            )
            .await
            .map_err(crate::map_jsonrpc_err)
    }

    async fn lock(&self, address: AccountAddress2) -> RpcResult<AccountInfo2> {
        let service = self.account.clone();
        service
            .lock_account(address)
            .await
            .map_err(crate::map_jsonrpc_err)
    }

    /// Import private key with address.
    async fn import(
        &self,
        address: AccountAddress2,
        private_key: StrView2<Vec<u8>>,
        password: String,
    ) -> RpcResult<AccountInfo2> {
        let service = self.account.clone();
        service
            .import_account(address, private_key.0, password)
            .await
            .map_err(crate::map_jsonrpc_err)
    }

    async fn import_readonly(
        &self,
        address: AccountAddress2,
        public_key: StrView2<Vec<u8>>,
    ) -> RpcResult<AccountInfo2> {
        let service = self.account.clone();
        service
            .import_readonly_account(address, public_key.0)
            .await
            .map_err(crate::map_jsonrpc_err)
    }

    /// Return the private key as bytes for `address`
    async fn export(&self, address: AccountAddress2, password: String) -> RpcResult<Vec<u8>> {
        let service = self.account.clone();
        service
            .export_account(address, password)
            .await
            .map_err(crate::map_jsonrpc_err)
    }

    async fn change_account_password(
        &self,
        address: AccountAddress2,
        new_password: String,
    ) -> RpcResult<AccountInfo2> {
        let account_service = self.account.clone();
        account_service
            .change_account_password(address, new_password)
            .await
            .map_err(crate::map_jsonrpc_err)
    }

    async fn accepted_tokens(&self, address: AccountAddress2) -> RpcResult<Vec<TokenCode2>> {
        let service = self.account.clone();
        service
            .accepted_tokens(address)
            .await
            .map_err(crate::map_jsonrpc_err)
    }

    async fn remove(
        &self,
        address: AccountAddress2,
        password: Option<String>,
    ) -> RpcResult<AccountInfo2> {
        let service = self.account.clone();
        service
            .remove_account(address, password)
            .await
            .map_err(crate::map_jsonrpc_err)
    }
}
