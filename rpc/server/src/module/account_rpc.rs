// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_account_api::{AccountAsyncService, AccountInfo};
use starcoin_rpc_api::{account::AccountApi, FutureResult};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};

pub struct AccountRpcImpl<S>
where
    S: AccountAsyncService + 'static,
{
    service: S,
}

impl<S> AccountRpcImpl<S>
where
    S: AccountAsyncService,
{
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

impl<S> AccountApi for AccountRpcImpl<S>
where
    S: AccountAsyncService,
{
    fn default(&self) -> FutureResult<Option<AccountInfo>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.get_default_account().await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn set_default_account(&self, addr: AccountAddress) -> FutureResult<Option<AccountInfo>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.set_default_account(addr).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn create(&self, password: String) -> FutureResult<AccountInfo> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.create_account(password).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn list(&self) -> FutureResult<Vec<AccountInfo>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.get_accounts().await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn get(&self, address: AccountAddress) -> FutureResult<Option<AccountInfo>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.get_account(address).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer: AccountAddress,
    ) -> FutureResult<SignedUserTransaction> {
        let service = self.service.clone();
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
        duration: std::time::Duration,
    ) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move { service.unlock_account(address, password, duration).await }
            .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn lock(&self, address: AccountAddress) -> FutureResult<()> {
        let service = self.service.clone();
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
        let service = self.service.clone();
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
        let service = self.service.clone();
        let fut = async move {
            let result = service.export_account(address, password).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn accepted_tokens(&self, address: AccountAddress) -> FutureResult<Vec<TokenCode>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.accepted_tokens(address).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpc_core::IoHandler;
    use starcoin_account_api::mock::MockAccountService;
    use starcoin_rpc_client::RpcClient;
    use tokio_compat::runtime::Runtime;

    #[test]
    fn test_account() {
        let mut io = IoHandler::new();
        let mut runtime = Runtime::new().unwrap();
        let account_service = MockAccountService::new().unwrap();
        io.extend_with(AccountRpcImpl::new(account_service).to_delegate());
        let client = RpcClient::connect_local(io, &mut runtime);
        let account = client.account_create("passwd".to_string()).unwrap();
        let accounts = client.account_list().unwrap();
        assert!(!accounts.is_empty());
        assert!(accounts.iter().any(|a| a.address() == account.address()));
        // assert!(accounts.contains(&account));
        let raw_txn = RawUserTransaction::mock_by_sender(account.address);
        let signed_txn = client.account_sign_txn(raw_txn).unwrap();
        assert!(signed_txn.check_signature().is_ok())
    }
}
