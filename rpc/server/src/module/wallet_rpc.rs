// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_rpc_err;
use futures::future::TryFutureExt;
use starcoin_rpc_api::{wallet::WalletApi, FutureResult};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_wallet_api::{WalletAccount, WalletAsyncService};

pub struct WalletRpcImpl<S>
where
    S: WalletAsyncService + 'static,
{
    service: S,
}

impl<S> WalletRpcImpl<S>
where
    S: WalletAsyncService,
{
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

impl<S> WalletApi for WalletRpcImpl<S>
where
    S: WalletAsyncService,
{
    fn default(&self) -> FutureResult<Option<WalletAccount>> {
        let fut = self
            .service
            .clone()
            .get_default_account()
            .map_err(|e| map_rpc_err(e.into()));
        Box::new(fut.compat())
    }

    fn create(&self, password: String) -> FutureResult<WalletAccount> {
        let fut = self
            .service
            .clone()
            .create_account(password)
            .map_err(|e| map_rpc_err(e.into()));
        Box::new(fut.compat())
    }

    fn list(&self) -> FutureResult<Vec<WalletAccount>> {
        let fut = self
            .service
            .clone()
            .get_accounts()
            .map_err(|e| map_rpc_err(e.into()));
        Box::new(fut.compat())
    }

    fn get(&self, address: AccountAddress) -> FutureResult<Option<WalletAccount>> {
        let fut = self
            .service
            .clone()
            .get_account(address)
            .map_err(|e| map_rpc_err(e.into()));
        Box::new(fut.compat())
    }

    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer: AccountAddress,
    ) -> FutureResult<SignedUserTransaction> {
        let fut = self
            .service
            .clone()
            .sign_txn(raw_txn, signer)
            .map_err(|e| map_rpc_err(e.into()));
        Box::new(fut.compat())
    }

    fn unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> FutureResult<()> {
        let fut = self
            .service
            .clone()
            .unlock_account(address, password, duration)
            .map_err(|e| map_rpc_err(e.into()));
        Box::new(fut.compat())
    }
    /// Import private key with address.
    fn import(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> FutureResult<WalletAccount> {
        let fut = self
            .service
            .clone()
            .import_account(address, private_key, password)
            .map_err(|e| map_rpc_err(e.into()));
        Box::new(fut.compat())
    }

    /// Return the private key as bytes for `address`
    fn export(&self, address: AccountAddress, password: String) -> FutureResult<Vec<u8>> {
        let fut = self
            .service
            .clone()
            .export_account(address, password)
            .map_err(|e| map_rpc_err(e.into()));
        Box::new(fut.compat())
    }

    fn accepted_tokens(&self, address: AccountAddress) -> FutureResult<Vec<TokenCode>> {
        let fut = self
            .service
            .clone()
            .accepted_tokens(address)
            .map_err(|e| map_rpc_err(e.into()));
        Box::new(fut.compat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpc_core::IoHandler;
    use starcoin_rpc_client::RpcClient;
    use starcoin_wallet_api::mock::MockWalletService;
    use tokio_compat::runtime::Runtime;

    #[test]
    fn test_account() {
        let mut io = IoHandler::new();
        let mut runtime = Runtime::new().unwrap();
        let wallet_service = MockWalletService::new().unwrap();
        io.extend_with(WalletRpcImpl::new(wallet_service).to_delegate());
        let client = RpcClient::connect_local(io, &mut runtime);
        let account = client.wallet_create("passwd".to_string()).unwrap();
        let accounts = client.wallet_list().unwrap();
        assert!(!accounts.is_empty());
        assert!(accounts.iter().any(|a| a.address() == account.address()));
        // assert!(accounts.contains(&account));
        let raw_txn = RawUserTransaction::mock_by_sender(account.address);
        let signed_txn = client.wallet_sign_txn(raw_txn).unwrap();
        assert!(signed_txn.check_signature().is_ok())
    }
}
