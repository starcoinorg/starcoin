// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::{WalletRequest, WalletResponse};
use actix::{Actor, Addr, Context, Handler};
use anyhow::Result;
use starcoin_bus::BusActor;
use starcoin_config::NodeConfig;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_wallet_api::error::AccountServiceError;
use starcoin_wallet_api::{ServiceResult, WalletAccount, WalletAsyncService, WalletResult};
use starcoin_wallet_lib::wallet_events::WalletEventActor;
use starcoin_wallet_lib::wallet_manager::WalletManager;
use starcoin_wallet_lib::wallet_storage::WalletStorage;
use std::sync::Arc;

pub struct WalletActor {
    service: WalletManager,
    _events: Addr<WalletEventActor>,
}

impl WalletActor {
    pub fn launch(config: Arc<NodeConfig>, bus: Addr<BusActor>) -> Result<WalletActorRef> {
        let vault_config = &config.vault;
        let wallet_storage = WalletStorage::create_from_path(vault_config.dir())?;
        let manager = WalletManager::new(wallet_storage.clone())?;
        let events = WalletEventActor::launch(bus, wallet_storage);
        let actor = WalletActor {
            service: manager,
            _events: events,
        };
        Ok(WalletActorRef(actor.start()))
    }
}

impl Actor for WalletActor {
    type Context = Context<Self>;
}

impl Handler<WalletRequest> for WalletActor {
    type Result = WalletResult<WalletResponse>;

    fn handle(&mut self, msg: WalletRequest, _ctx: &mut Self::Context) -> Self::Result {
        let response = match msg {
            WalletRequest::CreateAccount(password) => WalletResponse::WalletAccount(Box::new(
                self.service.create_wallet(password.as_str())?.wallet_info(),
            )),
            WalletRequest::GetDefaultAccount() => {
                WalletResponse::WalletAccountOption(Box::new(self.service.default_wallet_info()?))
            }
            WalletRequest::GetAccounts() => {
                WalletResponse::AccountList(self.service.list_wallet_infos()?)
            }
            WalletRequest::GetAccount(address) => {
                WalletResponse::WalletAccountOption(Box::new(self.service.wallet_info(address)?))
            }
            WalletRequest::SignTxn {
                txn: raw_txn,
                signer,
            } => WalletResponse::SignedTxn(Box::new(self.service.sign_txn(signer, *raw_txn)?)),
            WalletRequest::UnlockAccount(address, password, duration) => {
                self.service
                    .unlock_wallet(address, password.as_str(), duration)?;
                WalletResponse::UnlockAccountResponse
            }
            WalletRequest::LockAccount(address) => {
                self.service.lock_wallet(address)?;
                WalletResponse::None
            }
            WalletRequest::ExportAccount { address, password } => {
                let data = self.service.export_wallet(address, password.as_str())?;
                WalletResponse::ExportAccountResponse(data)
            }
            WalletRequest::ImportAccount {
                address,
                password,
                private_key,
            } => {
                let wallet = self
                    .service
                    .import_wallet(address, private_key, password.as_str())?;
                WalletResponse::WalletAccount(Box::new(wallet.wallet_info()))
            }
            WalletRequest::AccountAcceptedTokens { address } => {
                let tokens = self.service.accepted_tokens(address)?;
                WalletResponse::AcceptedTokens(tokens)
            }
        };
        Ok(response)
    }
}

#[derive(Clone)]
pub struct WalletActorRef(pub Addr<WalletActor>);

impl Into<Addr<WalletActor>> for WalletActorRef {
    fn into(self) -> Addr<WalletActor> {
        self.0
    }
}

impl Into<WalletActorRef> for Addr<WalletActor> {
    fn into(self) -> WalletActorRef {
        WalletActorRef(self)
    }
}

#[async_trait::async_trait]
impl WalletAsyncService for WalletActorRef {
    async fn create_account(self, password: String) -> ServiceResult<WalletAccount> {
        let response = self
            .0
            .send(WalletRequest::CreateAccount(password))
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let WalletResponse::WalletAccount(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_default_account(self) -> ServiceResult<Option<WalletAccount>> {
        let response = self
            .0
            .send(WalletRequest::GetDefaultAccount())
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let WalletResponse::WalletAccountOption(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_accounts(self) -> ServiceResult<Vec<WalletAccount>> {
        let response = self
            .0
            .send(WalletRequest::GetAccounts())
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let WalletResponse::AccountList(accounts) = response {
            Ok(accounts)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_account(self, address: AccountAddress) -> ServiceResult<Option<WalletAccount>> {
        let response = self
            .0
            .send(WalletRequest::GetAccount(address))
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let WalletResponse::WalletAccountOption(account) = response {
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
            .send(WalletRequest::SignTxn {
                txn: Box::new(raw_txn),
                signer: signer_address,
            })
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let WalletResponse::SignedTxn(txn) = response {
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
            .send(WalletRequest::UnlockAccount(address, password, duration))
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let WalletResponse::UnlockAccountResponse = response {
            Ok(())
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn lock_account(self, address: AccountAddress) -> ServiceResult<()> {
        let response = self
            .0
            .send(WalletRequest::LockAccount(address))
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let WalletResponse::None = response {
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
    ) -> ServiceResult<WalletAccount> {
        let response = self
            .0
            .send(WalletRequest::ImportAccount {
                address,
                password,
                private_key,
            })
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let WalletResponse::WalletAccount(account) = response {
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
            .send(WalletRequest::ExportAccount { address, password })
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let WalletResponse::ExportAccountResponse(data) = response {
            Ok(data)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn accepted_tokens(self, address: AccountAddress) -> ServiceResult<Vec<TokenCode>> {
        let response = self
            .0
            .send(WalletRequest::AccountAcceptedTokens { address })
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let WalletResponse::AcceptedTokens(data) = response {
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
        let actor = WalletActor::launch(config, bus)?;
        let account = actor.get_default_account().await?;
        assert!(account.is_none());
        Ok(())
    }
}
