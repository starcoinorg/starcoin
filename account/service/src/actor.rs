// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_account_api::{
    message::{AccountRequest, AccountResponse},
    AccountResult,
};
use starcoin_account_lib::{account_storage::AccountStorage, AccountManager};
use starcoin_service_registry::{ActorService, ServiceContext, ServiceFactory, ServiceHandler};

pub struct AccountService {
    manager: AccountManager,
}

impl ActorService for AccountService {}

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
    ) -> AccountResult<AccountResponse> {
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
                let tokens = self.manager.accepted_tokens(address)?;
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
        let config = NodeConfig::random_for_test();
        let registry = RegistryService::launch();
        let vault_config = &config.vault;
        let account_storage = AccountStorage::create_from_path(vault_config.dir())?;
        registry.put_shared(account_storage).await?;
        let service_ref = registry.registry::<AccountService>().await?;
        let account = service_ref.get_default_account().await?;
        assert!(account.is_none());
        Ok(())
    }
}
