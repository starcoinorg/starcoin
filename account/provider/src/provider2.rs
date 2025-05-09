// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{rpc_provider2::AccountRpcProvider, ProviderFactory};
use anyhow::{anyhow, Result};
use starcoin_config::account_provider_config::G_ENV_PRIVATE_KEY;
use starcoin_rpc_client::RpcClient;
use starcoin_vm2_account_api::{AccountProvider, AccountProviderStrategy};
use starcoin_vm2_account_provider::{
    local_provider::AccountLocalProvider, private_key_provider::AccountPrivateKeyProvider,
};
use starcoin_vm2_types::genesis_config::ChainId;
use std::sync::Arc;

impl ProviderFactory {
    pub fn create_provider2(
        rpc_client: Arc<RpcClient>,
        chain_id: ChainId,
        config: &AccountProviderConfig,
    ) -> Result<Box<dyn AccountProvider>> {
        match config.get_strategy() {
            AccountProviderStrategy::RPC => Ok(Box::new(AccountRpcProvider::create(rpc_client))),
            AccountProviderStrategy::Local => match AccountLocalProvider::create(
                config
                    .account_dir
                    .as_ref()
                    .ok_or_else(|| anyhow!("expect dir for local account"))?,
                chain_id,
            ) {
                Ok(p) => Ok(Box::new(p)),
                Err(e) => Err(e),
            },
            AccountProviderStrategy::PrivateKey => match AccountPrivateKeyProvider::create(
                config.secret_file.clone(),
                config.account_address,
                config.from_env,
                chain_id,
                G_ENV_PRIVATE_KEY.to_string(),
            ) {
                Ok(p) => Ok(Box::new(p)),
                Err(e) => Err(e),
            },
        }
    }
}
