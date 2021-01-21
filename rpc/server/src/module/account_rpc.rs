// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::helpers::TransactionRequestFiller;
use crate::module::map_err;
use failure::_core::time::Duration;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_account_api::{AccountAsyncService, AccountInfo};
use starcoin_config::NodeConfig;
use starcoin_rpc_api::types::{StrView, TransactionRequest};
use starcoin_rpc_api::{account::AccountApi, FutureResult};
use starcoin_state_api::ChainStateAsyncService;
use starcoin_traits::ChainAsyncService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::sync::Arc;

#[derive(Clone)]
pub struct AccountRpcImpl<Account, Pool, State, Chain>
where
    Account: AccountAsyncService + 'static,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
    Chain: ChainAsyncService + 'static,
{
    account: Account,
    pool: Pool,
    chain_state: State,
    chain: Chain,
    node_config: Arc<NodeConfig>,
}

impl<Account, Pool, State, Chain> AccountRpcImpl<Account, Pool, State, Chain>
where
    Account: AccountAsyncService,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
    Chain: ChainAsyncService + 'static,
{
    pub fn new(
        node_config: Arc<NodeConfig>,
        account: Account,
        pool: Pool,
        chain_state: State,
        chain: Chain,
    ) -> Self {
        Self {
            account,
            pool,
            chain_state,
            chain,
            node_config,
        }
    }
    fn txn_request_filler(&self) -> TransactionRequestFiller<Account, Pool, State, Chain> {
        TransactionRequestFiller {
            account: Some(self.account.clone()),
            pool: self.pool.clone(),
            chain_state: self.chain_state.clone(),
            chain: self.chain.clone(),
            node_config: self.node_config.clone(),
        }
    }
}

impl<S, Pool, State, Chain> AccountApi for AccountRpcImpl<S, Pool, State, Chain>
where
    S: AccountAsyncService,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
    Chain: ChainAsyncService + 'static,
{
    fn default(&self) -> FutureResult<Option<AccountInfo>> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.get_default_account().await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn set_default_account(&self, addr: AccountAddress) -> FutureResult<Option<AccountInfo>> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.set_default_account(addr).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn create(&self, password: String) -> FutureResult<AccountInfo> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.create_account(password).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn list(&self) -> FutureResult<Vec<AccountInfo>> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.get_accounts().await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn get(&self, address: AccountAddress) -> FutureResult<Option<AccountInfo>> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.get_account(address).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }
    fn sign(
        &self,
        address: AccountAddress,
        data: StrView<Vec<u8>>,
    ) -> FutureResult<StrView<Vec<u8>>> {
        let account_service = self.account.clone();
        let f = async move {
            let signature = account_service.sign_message(address, data.0).await?;
            Ok(signature.into())
        };
        Box::new(f.map_err(map_err).boxed().compat())
    }

    fn sign_txn_request(&self, txn_request: TransactionRequest) -> FutureResult<String> {
        let me = self.clone();
        let fut = async move {
            let raw_txn = me
                .txn_request_filler()
                .fill_transaction(txn_request)
                .await?;
            let sender = raw_txn.sender();
            let signed_txn = me.account.sign_txn(raw_txn, sender).await?;
            Ok(format!(
                "0x{}",
                hex::encode(bcs_ext::to_bytes(&signed_txn)?)
            ))
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer: AccountAddress,
    ) -> FutureResult<SignedUserTransaction> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.sign_txn(raw_txn, signer).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: Option<u32>,
    ) -> FutureResult<()> {
        let service = self.account.clone();
        let fut = async move {
            service
                .unlock_account(
                    address,
                    password,
                    Duration::from_secs(duration.unwrap_or_else(u32::max_value) as u64),
                )
                .await
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn lock(&self, address: AccountAddress) -> FutureResult<()> {
        let service = self.account.clone();
        let fut = async move { service.lock_account(address).await }.map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    /// Import private key with address.
    fn import(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> FutureResult<AccountInfo> {
        let service = self.account.clone();
        let fut = async move {
            let result = service
                .import_account(address, private_key, password)
                .await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    /// Return the private key as bytes for `address`
    fn export(&self, address: AccountAddress, password: String) -> FutureResult<Vec<u8>> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.export_account(address, password).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn change_account_password(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> FutureResult<()> {
        let account_service = self.account.clone();
        let fut = async move {
            Ok(account_service
                .change_account_password(address, new_password)
                .await?)
        };
        Box::new(fut.map_err(map_err).boxed().compat())
    }

    fn accepted_tokens(&self, address: AccountAddress) -> FutureResult<Vec<TokenCode>> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.accepted_tokens(address).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }
}
