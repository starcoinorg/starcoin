// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::TryFutureExt;
use starcoin_rpc_api::{account::AccountApi, FutureResult};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_wallet_api::{WalletAccount, WalletAsyncService};

pub struct AccountRpcImpl<S>
where
    S: WalletAsyncService + 'static,
{
    service: S,
}

impl<S> AccountRpcImpl<S>
where
    S: WalletAsyncService,
{
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

impl<S> AccountApi for AccountRpcImpl<S>
where
    S: WalletAsyncService,
{
    fn create(&self, password: String) -> FutureResult<WalletAccount> {
        let fut = self
            .service
            .clone()
            .create_account(password)
            .map_err(map_err);
        Box::new(fut.compat())
    }

    fn list(&self) -> FutureResult<Vec<WalletAccount>> {
        let fut = self.service.clone().get_accounts().map_err(map_err);
        Box::new(fut.compat())
    }

    fn get(&self, address: AccountAddress) -> FutureResult<Option<WalletAccount>> {
        let fut = self.service.clone().get_account(address).map_err(map_err);
        Box::new(fut.compat())
    }

    fn sign_txn(&self, raw_txn: RawUserTransaction) -> FutureResult<SignedUserTransaction> {
        let fut = self.service.clone().sign_txn(raw_txn).map_err(map_err);
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
            .map_err(map_err);
        Box::new(fut.compat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpc_core::IoHandler;
    use starcoin_rpc_client::RpcClient;
    use starcoin_wallet_api::mock::MockWalletService;

    #[test]
    fn test_account() {
        let mut io = IoHandler::new();
        let wallet_service = MockWalletService::new().unwrap();
        io.extend_with(AccountRpcImpl::new(wallet_service).to_delegate());
        let client = RpcClient::connect_local(io);
        let account = client.account_create("passwd".to_string()).unwrap();
        let accounts = client.account_list().unwrap();
        assert!(accounts.len() >= 1);
        assert!(accounts
            .iter()
            .find(|a| a.address() == account.address())
            .is_some());
        // assert!(accounts.contains(&account));
        let raw_txn = RawUserTransaction::mock_by_sender(account.address);
        let signed_txn = client.account_sign_txn(raw_txn).unwrap();
        assert!(signed_txn.check_signature().is_ok())
    }
}
