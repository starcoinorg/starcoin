// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_account_api::message::{AccountRequest, AccountResponse};
use starcoin_account_lib::{account_storage::AccountStorage, AccountManager};
use starcoin_config::NodeConfig;
use starcoin_crypto::ValidCryptoMaterial;
use starcoin_logger::prelude::*;
use starcoin_service_registry::mocker::MockHandler;
use starcoin_service_registry::{ActorService, ServiceContext, ServiceFactory, ServiceHandler};
use starcoin_types::account_config::{association_address, STC_TOKEN_CODE};
use std::any::Any;
use std::sync::Arc;

pub const DEFAULT_ACCOUNT_PASSWORD: &str = "";

pub struct AccountService {
    manager: AccountManager,
}

impl AccountService {
    pub fn mock() -> Result<Self> {
        let manager = AccountManager::new(AccountStorage::mock())?;
        //auto create default account.
        manager.create_account("")?;
        Ok(Self { manager })
    }
}

impl MockHandler<AccountService> for AccountService {
    fn handle(
        &mut self,
        r: Box<dyn Any>,
        ctx: &mut ServiceContext<AccountService>,
    ) -> Box<dyn Any> {
        let request = r
            .downcast::<AccountRequest>()
            .expect("Downcast to AccountRequest fail.");
        let result = ServiceHandler::<AccountService, AccountRequest>::handle(self, *request, ctx);
        Box::new(result)
    }
}

impl ActorService for AccountService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        let account = self.manager.default_account_info()?;

        if account.is_none() {
            self.manager.create_account(DEFAULT_ACCOUNT_PASSWORD)?;
        }

        let config = ctx
            .get_shared::<Arc<NodeConfig>>()
            .expect("Get NodeConfig should success.");

        //Only test/dev network association_key_pair contains private_key.
        if let (Some(association_private_key), _) =
            &config.net().genesis_config().association_key_pair
        {
            let association_account = self.manager.account_info(association_address())?;
            if association_account.is_none() {
                if let Err(e) = self.manager.import_account(
                    association_address(),
                    association_private_key.to_bytes().to_vec(),
                    "",
                ) {
                    error!("Import association account error:{:?}", e)
                } else {
                    info!("Import association account to wallet.");
                }
            }
        }
        Ok(())
    }
}

impl ServiceFactory<AccountService> for AccountService {
    fn create(ctx: &mut ServiceContext<AccountService>) -> Result<AccountService> {
        let account_storage = ctx.get_shared::<AccountStorage>()?;
        let manager = AccountManager::new(account_storage)?;
        Ok(Self { manager })
    }
}

impl ServiceHandler<AccountService, AccountRequest> for AccountService {
    fn handle(
        &mut self,
        msg: AccountRequest,
        _ctx: &mut ServiceContext<Self>,
    ) -> Result<AccountResponse> {
        let response = match msg {
            AccountRequest::CreateAccount(password) => AccountResponse::AccountInfo(Box::new(
                self.manager.create_account(password.as_str())?.info(),
            )),
            AccountRequest::GetDefaultAccount() => {
                AccountResponse::AccountInfoOption(Box::new(self.manager.default_account_info()?))
            }
            AccountRequest::SetDefaultAccount(address) => {
                let account_info = self.manager.account_info(address)?;

                // only set default if this address exists
                if account_info.is_some() {
                    self.manager.set_default_account(address)?;
                }
                AccountResponse::AccountInfoOption(Box::new(account_info))
            }
            AccountRequest::GetAccounts() => {
                AccountResponse::AccountList(self.manager.list_account_infos()?)
            }
            AccountRequest::GetAccount(address) => {
                AccountResponse::AccountInfoOption(Box::new(self.manager.account_info(address)?))
            }
            AccountRequest::SignTxn {
                txn: raw_txn,
                signer,
            } => AccountResponse::SignedTxn(Box::new(self.manager.sign_txn(signer, *raw_txn)?)),
            AccountRequest::UnlockAccount(address, password, duration) => {
                self.manager
                    .unlock_account(address, password.as_str(), duration)?;
                AccountResponse::UnlockAccountResponse
            }
            AccountRequest::LockAccount(address) => {
                self.manager.lock_account(address)?;
                AccountResponse::None
            }
            AccountRequest::ExportAccount { address, password } => {
                let data = self.manager.export_account(address, password.as_str())?;
                AccountResponse::ExportAccountResponse(data)
            }
            AccountRequest::ImportAccount {
                address,
                password,
                private_key,
            } => {
                let wallet =
                    self.manager
                        .import_account(address, private_key, password.as_str())?;
                AccountResponse::AccountInfo(Box::new(wallet.info()))
            }
            AccountRequest::AccountAcceptedTokens { address } => {
                let mut tokens = self.manager.accepted_tokens(address)?;
                //auto add STC to accepted tokens.
                if !tokens.contains(&STC_TOKEN_CODE) {
                    tokens.push(STC_TOKEN_CODE.clone())
                }
                AccountResponse::AcceptedTokens(tokens)
            }
        };
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_account_api::AccountAsyncService;
    use starcoin_config::NodeConfig;
    use starcoin_service_registry::{RegistryAsyncService, RegistryService};

    #[stest::test]
    async fn test_actor_launch() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let registry = RegistryService::launch();
        let vault_config = &config.vault;
        let account_storage = AccountStorage::create_from_path(vault_config.dir())?;
        registry.put_shared(config).await?;
        registry.put_shared(account_storage).await?;
        let service_ref = registry.register::<AccountService>().await?;
        let account = service_ref.get_default_account().await?;
        //default account will auto create
        assert!(account.is_some());
        Ok(())
    }
}
