// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::{WalletRequest, WalletResponse};
use crate::service::WalletServiceImpl;
use actix::{Actor, Addr, Context, Handler};
use anyhow::{Error, Result};
use starcoin_config::NodeConfig;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_wallet_api::mock::KeyPairWallet;
use starcoin_wallet_api::{Wallet, WalletAccount, WalletAsyncService};
use std::sync::Arc;

pub struct WalletActor {
    service: WalletServiceImpl,
}

impl WalletActor {
    pub fn launch(_config: Arc<NodeConfig>) -> Result<WalletActorRef> {
        let wallet = Arc::new(KeyPairWallet::new()?);
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
    type Result = Result<WalletResponse>;

    fn handle(&mut self, msg: WalletRequest, _ctx: &mut Self::Context) -> Self::Result {
        let response = match msg {
            WalletRequest::CreateAccount(password) => {
                WalletResponse::WalletAccount(self.service.create_account(password.as_str())?)
            }
            WalletRequest::GetDefaultAccount() => {
                WalletResponse::WalletAccountOption(self.service.get_default_account()?)
            }

            WalletRequest::GetAccounts() => {
                WalletResponse::AccountList(self.service.get_accounts()?)
            }

            WalletRequest::SignTxn(raw_txn) => {
                WalletResponse::SignedTxn(self.service.sign_txn(raw_txn)?)
            }
        };
        return Ok(response);
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

#[async_trait::async_trait(? Send)]
impl WalletAsyncService for WalletActorRef {
    async fn create_account(self, password: &str) -> Result<WalletAccount> {
        let response = self
            .0
            .send(WalletRequest::CreateAccount(password.to_string()))
            .await
            .map_err(|e| Into::<Error>::into(e))??;
        if let WalletResponse::WalletAccount(account) = response {
            Ok(account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_default_account(self) -> Result<Option<WalletAccount>> {
        let response = self
            .0
            .send(WalletRequest::GetDefaultAccount())
            .await
            .map_err(|e| Into::<Error>::into(e))??;
        if let WalletResponse::WalletAccountOption(account) = response {
            Ok(account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_accounts(self) -> Result<Vec<WalletAccount>> {
        let response = self
            .0
            .send(WalletRequest::GetAccounts())
            .await
            .map_err(|e| Into::<Error>::into(e))??;
        if let WalletResponse::AccountList(accounts) = response {
            Ok(accounts)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn sign_txn(self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        let response = self
            .0
            .send(WalletRequest::SignTxn(raw_txn))
            .await
            .map_err(|e| Into::<Error>::into(e))??;
        if let WalletResponse::SignedTxn(txn) = response {
            Ok(txn)
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
        let actor = WalletActor::launch(config)?;
        let account = actor.get_default_account().await?;
        assert!(account.is_some());
        Ok(())
    }
}
