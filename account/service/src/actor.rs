// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_events::AccountEventActor;
use actix::{Actor, ActorContext, Addr, Context, Handler};
use anyhow::Result;
use starcoin_account_api::{
    message::{AccountRequest, AccountResponse},
    AccountResult,
};
use starcoin_account_lib::{account_storage::AccountStorage, AccountManager};
use starcoin_logger::prelude::*;
use starcoin_node_api::service_registry::{ServiceRegistry, SystemService};
use starcoin_types::system_events::ActorStop;

pub struct AccountServiceActor {
    service: AccountManager,
    events: Addr<AccountEventActor>,
}

impl AccountServiceActor {
    pub fn new(registry: &ServiceRegistry) -> Result<AccountServiceActor> {
        let config = registry.config();
        let bus = registry.bus();
        let vault_config = &config.vault;
        let account_storage = AccountStorage::create_from_path(vault_config.dir())?;
        let manager = AccountManager::new(account_storage.clone())?;
        //TODO should registry account event actor?
        let events = AccountEventActor::launch(bus, account_storage);
        Ok(AccountServiceActor {
            service: manager,
            events,
        })
    }
}

impl Actor for AccountServiceActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Service {} started", Self::service_name());
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Service {} stopped", Self::service_name());
        self.events.do_send(ActorStop);
    }
}

impl SystemService for AccountServiceActor {}

impl Handler<ActorStop> for AccountServiceActor {
    type Result = ();

    fn handle(&mut self, _msg: ActorStop, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop()
    }
}

impl Handler<AccountRequest> for AccountServiceActor {
    type Result = AccountResult<AccountResponse>;

    fn handle(&mut self, msg: AccountRequest, _ctx: &mut Self::Context) -> Self::Result {
        let response = match msg {
            AccountRequest::CreateAccount(password) => AccountResponse::AccountInfo(Box::new(
                self.service.create_account(password.as_str())?.info(),
            )),
            AccountRequest::GetDefaultAccount() => {
                AccountResponse::AccountInfoOption(Box::new(self.service.default_account_info()?))
            }
            AccountRequest::SetDefaultAccount(address) => {
                let account_info = self.service.account_info(address)?;

                // only set default if this address exists
                if account_info.is_some() {
                    self.service.set_default_account(address)?;
                }
                AccountResponse::AccountInfoOption(Box::new(account_info))
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
                AccountResponse::AccountInfo(Box::new(wallet.info()))
            }
            AccountRequest::AccountAcceptedTokens { address } => {
                let tokens = self.service.accepted_tokens(address)?;
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
    use starcoin_bus::BusActor;
    use starcoin_config::NodeConfig;
    use std::sync::Arc;

    #[stest::test]
    async fn test_actor_launch() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let bus = BusActor::launch();
        let address = {
            let registry = ServiceRegistry::new(config.clone(), bus.clone());
            registry.registry(AccountServiceActor::new).unwrap();
            registry.start::<AccountServiceActor>().unwrap()
        };
        let account = address.get_default_account().await?;
        assert!(account.is_none());
        Ok(())
    }
}
