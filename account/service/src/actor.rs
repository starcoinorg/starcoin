// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::{AccountRequest, AccountResponse};
use actix::{Actor, Addr, Context, Handler};
use anyhow::Result;
use starcoin_account_api::{
    error::AccountServiceError, AccountAsyncService, AccountInfo, AccountResult, ServiceResult,
};
use starcoin_account_lib::{account_storage::AccountStorage, AccountEventActor, AccountManager};
use starcoin_bus::BusActor;
use starcoin_config::NodeConfig;
use starcoin_types::{
    account_address::AccountAddress,
    account_config::token_code::TokenCode,
    transaction::{RawUserTransaction, SignedUserTransaction},
};
use std::sync::Arc;

pub struct AccountServiceActor {
    service: AccountManager,
    _events: Addr<AccountEventActor>,
}

impl AccountServiceActor {
    pub fn launch(config: Arc<NodeConfig>, bus: Addr<BusActor>) -> Result<AccountServiceRef> {
        let vault_config = &config.vault;
        let account_storage = AccountStorage::create_from_path(vault_config.dir())?;
        let manager = AccountManager::new(account_storage.clone())?;
        let events = AccountEventActor::launch(bus, account_storage);
        let actor = AccountServiceActor {
            service: manager,
            _events: events,
        };
        Ok(AccountServiceRef(actor.start()))
    }
}

impl Actor for AccountServiceActor {
    type Context = Context<Self>;
}

impl Handler<AccountRequest> for AccountServiceActor {
    type Result = AccountResult<AccountResponse>;

    fn handle(&mut self, msg: AccountRequest, _ctx: &mut Self::Context) -> Self::Result {
        let response = match msg {
            AccountRequest::CreateAccount(password) => AccountResponse::AccountInfo(Box::new(
                self.service
                    .create_account(password.as_str())?
                    .wallet_info(),
            )),
            AccountRequest::GetDefaultAccount() => {
                AccountResponse::AccountInfoOption(Box::new(self.service.default_account_info()?))
            }
            AccountRequest::GetAccounts() => {
                AccountResponse::AccountList(self.service.list_account_infos()?)
            }
            AccountRequest::GetAccount(address) => {
                AccountResponse::AccountInfoOption(Box::new(self.service.account_info(address)?))
            }
            AccountRequest::SignTxn {
                txn: raw_txn,
                signer,
            } => AccountResponse::SignedTxn(Box::new(self.service.sign_txn(signer, *raw_txn)?)),
            AccountRequest::UnlockAccount(address, password, duration) => {
                self.service
                    .unlock_account(address, password.as_str(), duration)?;
                AccountResponse::UnlockAccountResponse
            }
            AccountRequest::LockAccount(address) => {
                self.service.lock_account(address)?;
                AccountResponse::None
            }
            AccountRequest::ExportAccount { address, password } => {
                let data = self.service.export_account(address, password.as_str())?;
                AccountResponse::ExportAccountResponse(data)
            }
            AccountRequest::ImportAccount {
                address,
                password,
                private_key,
            } => {
                let wallet =
                    self.service
                        .import_account(address, private_key, password.as_str())?;
                AccountResponse::AccountInfo(Box::new(wallet.wallet_info()))
            }
            AccountRequest::AccountAcceptedTokens { address } => {
                let tokens = self.service.accepted_tokens(address)?;
                AccountResponse::AcceptedTokens(tokens)
            }
        };
        Ok(response)
    }
}

#[derive(Clone)]
pub struct AccountServiceRef(pub Addr<AccountServiceActor>);

impl Into<Addr<AccountServiceActor>> for AccountServiceRef {
    fn into(self) -> Addr<AccountServiceActor> {
        self.0
    }
}

impl Into<AccountServiceRef> for Addr<AccountServiceActor> {
    fn into(self) -> AccountServiceRef {
        AccountServiceRef(self)
    }
}

#[async_trait::async_trait]
impl AccountAsyncService for AccountServiceRef {
    async fn create_account(self, password: String) -> ServiceResult<AccountInfo> {
        let response = self
            .0
            .send(AccountRequest::CreateAccount(password))
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::AccountInfo(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_default_account(self) -> ServiceResult<Option<AccountInfo>> {
        let response = self
            .0
            .send(AccountRequest::GetDefaultAccount())
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::AccountInfoOption(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_accounts(self) -> ServiceResult<Vec<AccountInfo>> {
        let response = self
            .0
            .send(AccountRequest::GetAccounts())
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::AccountList(accounts) = response {
            Ok(accounts)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_account(self, address: AccountAddress) -> ServiceResult<Option<AccountInfo>> {
        let response = self
            .0
            .send(AccountRequest::GetAccount(address))
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::AccountInfoOption(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn sign_txn(
        self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> ServiceResult<SignedUserTransaction> {
        let response = self
            .0
            .send(AccountRequest::SignTxn {
                txn: Box::new(raw_txn),
                signer: signer_address,
            })
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::SignedTxn(txn) = response {
            Ok(*txn)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn unlock_account(
        self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> ServiceResult<()> {
        let response = self
            .0
            .send(AccountRequest::UnlockAccount(address, password, duration))
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::UnlockAccountResponse = response {
            Ok(())
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn lock_account(self, address: AccountAddress) -> ServiceResult<()> {
        let response = self
            .0
            .send(AccountRequest::LockAccount(address))
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::None = response {
            Ok(())
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn import_account(
        self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> ServiceResult<AccountInfo> {
        let response = self
            .0
            .send(AccountRequest::ImportAccount {
                address,
                password,
                private_key,
            })
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::AccountInfo(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn export_account(
        self,
        address: AccountAddress,
        password: String,
    ) -> ServiceResult<Vec<u8>> {
        let response = self
            .0
            .send(AccountRequest::ExportAccount { address, password })
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::ExportAccountResponse(data) = response {
            Ok(data)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn accepted_tokens(self, address: AccountAddress) -> ServiceResult<Vec<TokenCode>> {
        let response = self
            .0
            .send(AccountRequest::AccountAcceptedTokens { address })
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::AcceptedTokens(data) = response {
            Ok(data)
        } else {
            panic!("Unexpect response type.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[stest::test]
    async fn test_actor_launch() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let bus = BusActor::launch();
        let actor = AccountServiceActor::launch(config, bus)?;
        let account = actor.get_default_account().await?;
        assert!(account.is_none());
        Ok(())
    }
}
