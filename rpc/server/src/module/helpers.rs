use starcoin_account_api::AccountAsyncService;
use starcoin_config::NodeConfig;
use starcoin_rpc_api::types::{ByteCodeOrScriptName, ScriptData, TransactionRequest};
use starcoin_state_api::ChainStateAsyncService;
use starcoin_traits::ChainAsyncService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::account_config::AccountResource;
use starcoin_types::genesis_config::StdlibVersion;
use starcoin_types::transaction::{
    Module, Package, RawUserTransaction, Script, TransactionPayload,
};
use std::str::FromStr;
use std::sync::Arc;
use stdlib::transaction_scripts::{StdlibScript, VersionedStdlibScript};

#[derive(Clone)]
pub(crate) struct TransactionRequestFiller<Account, Pool, State, Chain> {
    pub(crate) account: Option<Account>,
    pub(crate) pool: Pool,
    pub(crate) chain_state: State,
    pub(crate) chain: Chain,
    pub(crate) node_config: Arc<NodeConfig>,
}

impl<Account, Pool, State, Chain> TransactionRequestFiller<Account, Pool, State, Chain>
where
    Account: AccountAsyncService + 'static,
    Pool: TxPoolSyncService + 'static,
    State: ChainStateAsyncService + 'static,
    Chain: ChainAsyncService + 'static,
{
    pub(crate) async fn fill_transaction(
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
            let _script = txn_request
                .script
                .map(|script_data| build_script(stdlib_version, script_data))
                .transpose()?;
            //TODO support SciptFunction
            TransactionPayload::Package(Package::new(modules, None)?)
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
            None => match self.account.as_ref() {
                None => anyhow::bail!("sender "),
                Some(account_service) => {
                    account_service
                        .get_default_account()
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("cannot find default account"))?
                        .address
                }
            },
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
        let max_gas_amount = txn_request.max_gas_amount.unwrap_or(1000000); // default 10_00000
        let max_gas_price = txn_request.gas_unit_price.unwrap_or(1);
        let expire = txn_request
            .expiration_timestamp_secs
            .unwrap_or_else(|| self.node_config.net().time_service().now_secs() + 60 * 60 * 12); // default to 0.5d

        let chain_id = self.chain.main_status().await?.head().chain_id();
        if let Some(cid) = txn_request.chain_id {
            if cid != chain_id.id() {
                anyhow::bail!(
                    "invalid transaction request: chain id mismatch, expected: {}, actual: {}",
                    chain_id.id(),
                    cid
                );
            }
        }

        let raw_txn = RawUserTransaction::new_with_default_gas_token(
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
