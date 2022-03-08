// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{ExecuteResultView, ExecutionOutputView, TransactionOptions};
use anyhow::{bail, format_err, Result};
use bcs_ext::BCSCodec;
use serde::de::DeserializeOwned;
use starcoin_abi_decoder::{decode_txn_payload, DecodedTransactionPayload};
use starcoin_account_api::{AccountInfo, AccountProvider};
use starcoin_config::{ChainNetworkID, DataDirPath};
use starcoin_crypto::HashValue;
use starcoin_node::NodeHandle;
use starcoin_rpc_api::chain::GetEventOption;
use starcoin_rpc_api::types::{RawUserTransactionView, TransactionStatusView};
use starcoin_rpc_client::{RpcClient, StateRootOption};
use starcoin_state_api::StateReaderExt;
use starcoin_types::account_config::AccountResource;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::association_address;
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::token::stc::STC_TOKEN_CODE_STR;
use starcoin_vm_types::transaction::{DryRunTransaction, RawUserTransaction, TransactionPayload};
use std::convert::TryInto;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

static HISTORY_FILE_NAME: &str = "history";

pub struct CliState {
    net: ChainNetworkID,
    client: Arc<RpcClient>,
    watch_timeout: Duration,
    node_handle: Option<NodeHandle>,
    /// Cli data dir, different with Node data dir.
    data_dir: PathBuf,
    temp_dir: DataDirPath,
    account_client: Box<dyn AccountProvider>,
}

impl CliState {
    pub const DEFAULT_WATCH_TIMEOUT: Duration = Duration::from_secs(300);
    pub const DEFAULT_MAX_GAS_AMOUNT: u64 = 10000000;
    pub const DEFAULT_GAS_PRICE: u64 = 1;
    pub const DEFAULT_EXPIRATION_TIME_SECS: u64 = 3600;
    pub const DEFAULT_GAS_TOKEN: &'static str = STC_TOKEN_CODE_STR;

    pub fn new(
        net: ChainNetworkID,
        client: Arc<RpcClient>,
        watch_timeout: Option<Duration>,
        node_handle: Option<NodeHandle>,
        account_client: Box<dyn AccountProvider>,
    ) -> CliState {
        let data_dir = starcoin_config::DEFAULT_BASE_DATA_DIR
            .clone()
            .join("cli")
            .join(net.to_string());
        if !data_dir.exists() {
            std::fs::create_dir_all(data_dir.as_path())
                .unwrap_or_else(|e| panic!("Create cli data dir {:?} fail, err:{:?}", data_dir, e))
        }
        let temp_dir = data_dir.join("tmp");
        if !temp_dir.exists() {
            std::fs::create_dir_all(temp_dir.as_path())
                .unwrap_or_else(|e| panic!("Create cli temp dir {:?} fail, err:{:?}", temp_dir, e))
        }
        let temp_dir = starcoin_config::temp_dir_in(temp_dir);

        Self {
            net,
            client,
            watch_timeout: watch_timeout.unwrap_or(Self::DEFAULT_WATCH_TIMEOUT),
            node_handle,
            data_dir,
            temp_dir,
            account_client,
        }
    }

    pub fn net(&self) -> &ChainNetworkID {
        &self.net
    }

    pub fn client(&self) -> &RpcClient {
        &self.client
    }

    pub fn account_client(&self) -> &dyn AccountProvider {
        self.account_client.as_ref()
    }
    pub fn temp_dir(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Cli data_dir , ~/.starcoin/cli/$network
    pub fn data_dir(&self) -> &Path {
        self.data_dir.as_path()
    }

    pub fn history_file(&self) -> PathBuf {
        self.data_dir().join(HISTORY_FILE_NAME)
    }

    pub fn node_handle(&self) -> Option<&NodeHandle> {
        self.node_handle.as_ref()
    }

    pub fn default_account(&self) -> Result<AccountInfo> {
        self.account_client
            .get_default_account()?
            .ok_or_else(|| format_err!("Can not find default account, Please input from account."))
    }

    /// Get account from node managed wallet.
    pub fn get_account(&self, account_address: AccountAddress) -> Result<AccountInfo> {
        self.account_client
            .get_account(account_address)?
            .ok_or_else(|| {
                format_err!("Can not find WalletAccount by address: {}", account_address)
            })
    }

    pub fn get_account_or_default(
        &self,
        account_address: Option<AccountAddress>,
    ) -> Result<AccountInfo> {
        if let Some(account_address) = account_address {
            self.account_client
                .get_account(account_address)?
                .ok_or_else(|| {
                    format_err!("Can not find WalletAccount by address: {}", account_address)
                })
        } else {
            self.default_account()
        }
    }

    pub fn get_resource<R>(&self, address: AccountAddress) -> Result<Option<R>>
    where
        R: MoveResource + DeserializeOwned,
    {
        let chain_state_reader = self.client.state_reader(StateRootOption::Latest)?;
        chain_state_reader.get_resource::<R>(address)
    }

    pub fn get_account_resource(&self, address: AccountAddress) -> Result<Option<AccountResource>> {
        self.get_resource::<AccountResource>(address)
    }

    pub fn association_account(&self) -> Result<Option<AccountInfo>> {
        self.client.account_get(association_address())
    }

    pub fn watch_txn(&self, txn_hash: HashValue) -> Result<ExecutionOutputView> {
        let block = self.client.watch_txn(txn_hash, Some(self.watch_timeout))?;

        let txn_info = {
            if let Some(info) = self.client.chain_get_transaction_info(txn_hash)? {
                info
            } else {
                //sleep and try again.
                std::thread::sleep(Duration::from_secs(1));
                if let Some(info) = self.client.chain_get_transaction_info(txn_hash)? {
                    info
                } else {
                    bail!("transaction execute success, but get transaction info return none, block: {}", block.header.number);
                }
            }
        };
        let events = self
            .client
            .chain_get_events_by_txn_hash(txn_hash, Some(GetEventOption { decode: true }))?;

        Ok(ExecutionOutputView::new_with_info(
            txn_hash, txn_info, events,
        ))
    }

    pub fn build_and_execute_transaction(
        &self,
        txn_opts: TransactionOptions,
        payload: TransactionPayload,
    ) -> Result<ExecuteResultView> {
        let (raw_txn, future_transaction) = self.build_transaction(
            txn_opts.sender,
            txn_opts.sequence_number,
            txn_opts.gas_unit_price,
            txn_opts.max_gas_amount,
            txn_opts.expiration_time_secs,
            payload,
        )?;
        if future_transaction {
            //TODO figure out more graceful method to handle future transaction.
            bail!("there is transaction from sender({}) in the txpool, please wait it to been executed or use sequence_number({}) to replace it.",raw_txn.sender(), raw_txn.sequence_number()-1);
        }
        self.execute_transaction(raw_txn, txn_opts.dry_run, txn_opts.blocking)
    }

    fn build_transaction(
        &self,
        sender: Option<AccountAddress>,
        sequence_number: Option<u64>,
        gas_price: Option<u64>,
        max_gas_amount: Option<u64>,
        expiration_time_secs: Option<u64>,
        payload: TransactionPayload,
    ) -> Result<(RawUserTransaction, bool)> {
        let chain_id = self.net().chain_id();
        let sender = self.get_account_or_default(sender)?;
        let (sequence_number, future_transaction) = match sequence_number {
            Some(sequence_number) => (sequence_number, false),
            None => match self.client.next_sequence_number_in_txpool(sender.address)? {
                Some(sequence_number) => {
                    eprintln!("get sequence_number {} from txpool", sequence_number);
                    (sequence_number, true)
                }
                None => self
                    .get_account_resource(*sender.address())?
                    .map(|account| (account.sequence_number(), false))
                    .ok_or_else(|| {
                        format_err!(
                            "Can not find account on chain by address:{}",
                            sender.address()
                        )
                    })?,
            },
        };
        let node_info = self.client.node_info()?;
        let expiration_timestamp_secs = expiration_time_secs
            .unwrap_or(Self::DEFAULT_EXPIRATION_TIME_SECS)
            + node_info.now_seconds;
        Ok((
            RawUserTransaction::new(
                sender.address,
                sequence_number,
                payload,
                max_gas_amount.unwrap_or(Self::DEFAULT_MAX_GAS_AMOUNT),
                gas_price.unwrap_or(Self::DEFAULT_GAS_PRICE),
                expiration_timestamp_secs,
                chain_id,
                Self::DEFAULT_GAS_TOKEN.to_string(),
            ),
            future_transaction,
        ))
    }

    pub fn execute_transaction(
        &self,
        raw_txn: RawUserTransaction,
        only_dry_run: bool,
        blocking: bool,
    ) -> Result<ExecuteResultView> {
        let sender = self.get_account(raw_txn.sender())?;
        let dry_output = self.client.dry_run_raw(DryRunTransaction {
            public_key: sender.public_key,
            raw_txn: raw_txn.clone(),
        })?;
        let mut raw_txn_view: RawUserTransactionView = raw_txn.clone().try_into()?;
        raw_txn_view.decoded_payload =
            Some(self.decode_txn_payload(raw_txn.payload())?.try_into()?);

        let mut execute_result = ExecuteResultView::new(raw_txn_view, raw_txn.to_hex(), dry_output);
        if only_dry_run
            || !matches!(
                execute_result.dry_run_output.txn_output.status,
                TransactionStatusView::Executed
            )
        {
            eprintln!("txn dry run failed");
            return Ok(execute_result);
        }
        let signed_txn = self.account_client.sign_txn(raw_txn, sender.address)?;
        let signed_txn_hex = hex::encode(signed_txn.encode()?);
        let txn_hash = self.client.submit_hex_transaction(signed_txn_hex)?;
        eprintln!("txn {} submitted.", txn_hash);
        let execute_output = if blocking {
            self.watch_txn(txn_hash)?
        } else {
            ExecutionOutputView::new(txn_hash)
        };
        execute_result.execute_output = Some(execute_output);
        Ok(execute_result)
    }

    pub fn decode_txn_payload(
        &self,
        payload: &TransactionPayload,
    ) -> Result<DecodedTransactionPayload> {
        let chain_state_reader = self.client.state_reader(StateRootOption::Latest)?;
        decode_txn_payload(&chain_state_reader, payload)
    }

    pub fn into_inner(self) -> (ChainNetworkID, Arc<RpcClient>, Option<NodeHandle>) {
        (self.net, self.client, self.node_handle)
    }
}
