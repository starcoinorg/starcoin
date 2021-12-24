// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::helpers::TransactionRequestFiller;
use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_account_api::{AccountAsyncService, AccountInfo};

use starcoin_config::NodeConfig;
use starcoin_rpc_api::types::{SignedMessageView, StrView, TransactionRequest};
use starcoin_rpc_api::{account::AccountApi, FutureResult};
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

impl<S, Pool, State> AccountApi for AccountRpcImpl<S, Pool, State>
where
    S: AccountAsyncService,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
{
    fn default(&self) -> FutureResult<Option<AccountInfo>> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.get_default_account().await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn set_default_account(&self, addr: AccountAddress) -> FutureResult<AccountInfo> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.set_default_account(addr).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn create(&self, password: String) -> FutureResult<AccountInfo> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.create_account(password).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn list(&self) -> FutureResult<Vec<AccountInfo>> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.get_accounts().await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn get(&self, address: AccountAddress) -> FutureResult<Option<AccountInfo>> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.get_account(address).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }
    fn sign(
        &self,
        address: AccountAddress,
        data: SigningMessage,
    ) -> FutureResult<SignedMessageView> {
        let account_service = self.account.clone();
        let f = async move {
            let signed_message = account_service.sign_message(address, data).await?;
            Ok(signed_message.into())
        };
        Box::pin(f.map_err(map_err).boxed())
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
        Box::pin(fut.boxed())
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
        Box::pin(fut.boxed())
    }

    fn unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: Option<u32>,
    ) -> FutureResult<AccountInfo> {
        let service = self.account.clone();
        let fut = async move {
            service
                .unlock_account(
                    address,
                    password,
                    Duration::from_secs(duration.unwrap_or(u32::MAX) as u64),
                )
                .await
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn lock(&self, address: AccountAddress) -> FutureResult<AccountInfo> {
        let service = self.account.clone();
        let fut = async move { service.lock_account(address).await }.map_err(map_err);
        Box::pin(fut.boxed())
    }

    /// Import private key with address.
    fn import(
        &self,
        address: AccountAddress,
        private_key: StrView<Vec<u8>>,
        password: String,
    ) -> FutureResult<AccountInfo> {
        let service = self.account.clone();
        let fut = async move {
            let result = service
                .import_account(address, private_key.0, password)
                .await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn import_readonly(
        &self,
        address: AccountAddress,
        public_key: StrView<Vec<u8>>,
    ) -> FutureResult<AccountInfo> {
        let service = self.account.clone();
        let fut = async move {
            let result = service
                .import_readonly_account(address, public_key.0)
                .await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    /// Return the private key as bytes for `address`
    fn export(&self, address: AccountAddress, password: String) -> FutureResult<Vec<u8>> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.export_account(address, password).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn change_account_password(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> FutureResult<AccountInfo> {
        let account_service = self.account.clone();
        let fut = async move {
            account_service
                .change_account_password(address, new_password)
                .await
        };
        Box::pin(fut.map_err(map_err).boxed())
    }

    fn accepted_tokens(&self, address: AccountAddress) -> FutureResult<Vec<TokenCode>> {
        let service = self.account.clone();
        let fut = async move {
            let result = service.accepted_tokens(address).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn remove(
        &self,
        address: AccountAddress,
        password: Option<String>,
    ) -> FutureResult<AccountInfo> {
        let service = self.account.clone();
        let fut = async move { service.remove_account(address, password).await }.map_err(map_err);
        Box::pin(fut.boxed())
    }
}
