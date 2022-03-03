use crate::local_provider::AccountLocalProvider;
use crate::rpc_provider::AccountRpcProvider;
use anyhow::{anyhow, Result};
use starcoin_account_api::{AccountProvider, AccountProviderStrategy};
use starcoin_config::account_provider_config::AccountProviderConfig;
use starcoin_rpc_client::RpcClient;
use starcoin_types::genesis_config::ChainId;
use std::sync::Arc;

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create_provider(
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
                    .ok_or(anyhow!("expect dir for local account"))?,
                chain_id,
            ) {
                Ok(p) => Ok(Box::new(p)),
                Err(e) => Err(e),
            },
        }
    }
}
