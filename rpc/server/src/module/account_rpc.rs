// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use failure::_core::time::Duration;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_account_api::{AccountAsyncService, AccountInfo};
use starcoin_config::NodeConfig;
use starcoin_rpc_api::types::{ByteCodeOrScriptName, ScriptData, TransactionRequest};
use starcoin_rpc_api::{account::AccountApi, FutureResult};
use starcoin_state_api::ChainStateAsyncService;
use starcoin_traits::ChainAsyncService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::transaction::{Module, RawUserTransaction, SignedUserTransaction};
use starcoin_vm_types::account_config::AccountResource;
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::transaction::{Package, Script, TransactionPayload};
use std::str::FromStr;
use std::sync::Arc;
use stdlib::transaction_scripts::{StdlibScript, VersionedStdlibScript};

#[derive(Clone)]
pub struct AccountRpcImpl<S, Pool, State, Chain>
where
    S: AccountAsyncService + 'static,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
    Chain: ChainAsyncService + 'static,
{
    service: S,
    pool: Pool,
    chain_state: State,
    chain: Chain,
    node_config: Arc<NodeConfig>,
}

impl<S, Pool, State, Chain> AccountRpcImpl<S, Pool, State, Chain>
where
    S: AccountAsyncService,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
    Chain: ChainAsyncService + 'static,
{
    pub fn new(
        node_config: Arc<NodeConfig>,
        service: S,
        pool: Pool,
        chain_state: State,
        chain: Chain,
    ) -> Self {
        Self {
            service,
            pool,
            chain_state,
            chain,
            node_config,
        }
    }
}

impl<S, Pool, State, Chain> AccountRpcImpl<S, Pool, State, Chain>
where
    S: AccountAsyncService,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
    Chain: ChainAsyncService + 'static,
{
    async fn constrcut_raw_txn(
        &self,
        txn_request: TransactionRequest,
    ) -> anyhow::Result<RawUserTransaction> {
        let build_script = |stdlib_version: StdlibVersion, script_data: ScriptData| {
            let code = match script_data.code.0 {
                ByteCodeOrScriptName::ScriptName(script_name) => {
                    VersionedStdlibScript::new(stdlib_version)
                        .compiled_bytes(StdlibScript::from_str(script_name.as_str())?)
                        .into_vec()
                }
                ByteCodeOrScriptName::ByteCode(c) => c,
            };
            let ty_args: Vec<_> = script_data.type_args.into_iter().map(|s| s.0).collect();
            let args: Vec<_> = script_data.args.into_iter().map(|s| s.0).collect();
            Ok::<_, anyhow::Error>(Script::new(code, ty_args, args))
        };
        let stdlib_version = self.node_config.net().genesis_config().stdlib_version;
        let payload = if !txn_request.modules.is_empty() {
            let modules = txn_request
                .modules
                .into_iter()
                .map(|c| Module::new(c.0))
                .collect();
            let script = txn_request
                .script
                .map(|script_data| build_script(stdlib_version, script_data))
                .transpose()?;
            TransactionPayload::Package(Package::new(modules, script)?)
        } else {
            let script = txn_request.script.ok_or_else(|| {
                anyhow::anyhow!(
                    "invalid transaction request: script should not be empty if no modules"
                )
            })?;
            TransactionPayload::Script(build_script(stdlib_version, script)?)
        };

        let sender = match txn_request.sender {
            Some(s) => s,
            None => {
                self.service
                    .get_default_account()
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("cannot find default account"))?
                    .address
            }
        };
        let next_seq_number = match txn_request
            .sequence_number
            .or_else(|| self.pool.next_sequence_number(sender))
        {
            Some(n) => n,
            None => match self
                .chain_state
                .clone()
                .get_resource::<AccountResource>(sender)
                .await?
            {
                Some(r) => r.sequence_number(),
                None => anyhow::bail!("cannot find account {} onchain", sender),
            },
        };
        let max_gas_amount = txn_request.max_gas_amount.unwrap_or(100000); // default 10_0000
        let max_gas_price = txn_request.gas_unit_price.unwrap_or(1);
        let expire = txn_request
            .expiration_timestamp_secs
            .unwrap_or_else(|| self.node_config.net().time_service().now_secs() + 60 * 60 * 12); // default to 0.5d

        let chain_id = self.chain.main_status().await?.head().chain_id;
        if let Some(cid) = txn_request.chain_id {
            if cid != chain_id.id() {
                anyhow::bail!(
                    "invalid transaction request: chain id mismatch, expected: {}, actual: {}",
                    chain_id.id(),
                    cid
                );
            }
        }

        let raw_txn = RawUserTransaction::new(
            sender,
            next_seq_number,
            payload,
            max_gas_amount,
            max_gas_price,
            expire,
            chain_id,
        );
        Ok(raw_txn)
    }
}

impl<S, Pool, State, Chain> AccountApi for AccountRpcImpl<S, Pool, State, Chain>
where
    S: AccountAsyncService,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
    Chain: ChainAsyncService + 'static,
{
    fn default(&self) -> FutureResult<Option<AccountInfo>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.get_default_account().await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn set_default_account(&self, addr: AccountAddress) -> FutureResult<Option<AccountInfo>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.set_default_account(addr).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn create(&self, password: String) -> FutureResult<AccountInfo> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.create_account(password).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn list(&self) -> FutureResult<Vec<AccountInfo>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.get_accounts().await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn get(&self, address: AccountAddress) -> FutureResult<Option<AccountInfo>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.get_account(address).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn sign_txn_request(&self, txn_request: TransactionRequest) -> FutureResult<String> {
        let me = self.clone();
        let fut = async move {
            let raw_txn = me.constrcut_raw_txn(txn_request).await?;
            let sender = raw_txn.sender();
            let signed_txn = me.service.sign_txn(raw_txn, sender).await?;
            Ok(format!("0x{}", hex::encode(scs::to_bytes(&signed_txn)?)))
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer: AccountAddress,
    ) -> FutureResult<SignedUserTransaction> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.sign_txn(raw_txn, signer).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: Option<u32>,
    ) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move {
            service
                .unlock_account(
                    address,
                    password,
                    Duration::from_secs(duration.unwrap_or_else(|| u32::max_value()) as u64),
                )
                .await
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn lock(&self, address: AccountAddress) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move { service.lock_account(address).await }.map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    /// Import private key with address.
    fn import(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> FutureResult<AccountInfo> {
        let service = self.service.clone();
        let fut = async move {
            let result = service
                .import_account(address, private_key, password)
                .await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    /// Return the private key as bytes for `address`
    fn export(&self, address: AccountAddress, password: String) -> FutureResult<Vec<u8>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.export_account(address, password).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }

    fn accepted_tokens(&self, address: AccountAddress) -> FutureResult<Vec<TokenCode>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.accepted_tokens(address).await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::new(fut.boxed().compat())
    }
}
