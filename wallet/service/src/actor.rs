// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::{WalletRequest, WalletResponse};
use crate::service::WalletServiceImpl;
use actix::{Actor, Addr, Context, Handler};
use anyhow::Result;
use starcoin_config::NodeConfig;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_wallet_lib::{file_wallet_store::FileWalletStore, keystore_wallet::KeyStoreWallet};

use starcoin_wallet_api::error::AccountServiceError;
use starcoin_wallet_api::{ServiceResult, Wallet, WalletAccount, WalletAsyncService, WalletResult};
use std::sync::Arc;

pub struct WalletActor {
    service: WalletServiceImpl<KeyStoreWallet<FileWalletStore>>,
}

impl WalletActor {
    pub fn launch(config: Arc<NodeConfig>) -> Result<WalletActorRef> {
        let vault_config = &config.vault;
        let file_store = FileWalletStore::new(vault_config.dir());
        let wallet = KeyStoreWallet::new(file_store)?;
        let actor = WalletActor {
            service: WalletServiceImpl::new(wallet),
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
                self.service.create_account(password.as_str())?,
            )),
            WalletRequest::GetDefaultAccount() => {
                WalletResponse::WalletAccountOption(Box::new(self.service.get_default_account()?))
            }
            WalletRequest::GetAccounts() => {
                WalletResponse::AccountList(self.service.get_accounts()?)
            }
            WalletRequest::GetAccount(address) => {
                WalletResponse::WalletAccountOption(Box::new(self.service.get_account(&address)?))
            }
            WalletRequest::SignTxn {
                txn: raw_txn,
                signer,
            } => WalletResponse::SignedTxn(Box::new(self.service.sign_txn(*raw_txn, signer)?)),
            WalletRequest::UnlockAccount(address, password, duration) => {
                self.service
                    .unlock_account(address, password.as_str(), duration)?;
                WalletResponse::UnlockAccountResponse
            }
            WalletRequest::ExportAccount { address, password } => {
                let data = self.service.export_account(&address, password.as_str())?;
                WalletResponse::ExportAccountResponse(data)
            }
            WalletRequest::ImportAccount {
                address,
                password,
                private_key,
            } => {
                let account =
                    self.service
                        .import_account(address, private_key, password.as_str())?;
                WalletResponse::WalletAccount(Box::new(account))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_config::{BaseConfig, ChainNetwork, ConfigModule};

    #[stest::test]
    async fn test_actor_launch() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let base_config = BaseConfig::new(ChainNetwork::Dev, Some(temp_dir.path().to_path_buf()));
        std::fs::create_dir_all(base_config.data_dir())?;

        let mut node_config = NodeConfig::random_for_test();
        node_config.vault.random(&base_config);
        let config = Arc::new(node_config);
        let actor = WalletActor::launch(config)?;
        let account = actor.get_default_account().await?;
        assert!(account.is_none());
        Ok(())
    }
}
