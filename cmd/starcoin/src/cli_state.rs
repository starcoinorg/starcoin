// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{ExecuteResultView, ExecutionOutputView, TransactionOptions};
use anyhow::{format_err, Result};
use starcoin_account_api::AccountInfo;
use starcoin_config::{ChainNetworkID, DataDirPath};
use starcoin_crypto::HashValue;
use starcoin_dev::playground;
use starcoin_node::NodeHandle;
use starcoin_rpc_api::types::TransactionInfoView;
use starcoin_rpc_client::chain_watcher::ThinHeadBlock;
use starcoin_rpc_client::{RemoteStateReader, RpcClient};
use starcoin_state_api::StateReaderExt;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::association_address;
use starcoin_vm_types::token::stc::STC_TOKEN_CODE_STR;
use starcoin_vm_types::transaction::{
    DryRunTransaction, RawUserTransaction, TransactionPayload, TransactionStatus,
};
use starcoin_vm_types::vm_status::KeptVMStatus;
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
        let temp_dir = starcoin_config::temp_path_with_dir(temp_dir);

        Self {
            net,
            client,
            watch_timeout: watch_timeout.unwrap_or(Self::DEFAULT_WATCH_TIMEOUT),
            node_handle,
            data_dir,
            temp_dir,
        }
    }

    pub fn net(&self) -> &ChainNetworkID {
        &self.net
    }

    pub fn client(&self) -> &RpcClient {
        &self.client
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
        self.client
            .account_default()?
            .ok_or_else(|| format_err!("Can not find default account, Please input from account."))
    }

    /// Get account from node managed wallet.
    pub fn get_account(&self, account_address: AccountAddress) -> Result<AccountInfo> {
        self.client.account_get(account_address)?.ok_or_else(|| {
            format_err!("Can not find WalletAccount by address: {}", account_address)
        })
    }

    pub fn get_account_or_default(
        &self,
        account_address: Option<AccountAddress>,
    ) -> Result<AccountInfo> {
        if let Some(account_address) = account_address {
            self.client.account_get(account_address)?.ok_or_else(|| {
                format_err!("Can not find WalletAccount by address: {}", account_address)
            })
        } else {
            self.default_account()
        }
    }

    pub fn association_account(&self) -> Result<Option<AccountInfo>> {
        self.client.account_get(association_address())
    }

    pub fn watch_txn(
        &self,
        txn_hash: HashValue,
    ) -> Result<(ThinHeadBlock, Option<TransactionInfoView>)> {
        let block = self.client.watch_txn(txn_hash, Some(self.watch_timeout))?;

        let mut txn_info = self.client.chain_get_transaction_info(txn_hash)?;
        std::thread::sleep(Duration::from_secs(1));
        if txn_info.is_none() {
            txn_info = self.client.chain_get_transaction_info(txn_hash)?;
        }
        if txn_info.is_none() {
            eprintln!("transaction execute success, but get transaction info return none");
        }
        Ok((block, txn_info))
    }

    pub fn build_and_execute_transaction(
        &self,
        txn_opts: TransactionOptions,
        payload: TransactionPayload,
    ) -> Result<ExecuteResultView> {
        self.execute_transaction(
            self.build_transaction(
                txn_opts.sender,
                txn_opts.gas_price,
                txn_opts.max_gas_amount,
                txn_opts.expiration_time_secs,
                payload,
            )?,
            txn_opts.dry_run,
            txn_opts.blocking,
        )
    }

    pub fn build_transaction(
        &self,
        sender: Option<AccountAddress>,
        gas_price: Option<u64>,
        max_gas_amount: Option<u64>,
        expiration_time_secs: Option<u64>,
        payload: TransactionPayload,
    ) -> Result<RawUserTransaction> {
        let chain_id = self.net().chain_id();
        let sender = self.get_account_or_default(sender)?;
        let sequence_number = match self.client.next_sequence_number_in_txpool(sender.address)? {
            Some(sequence_number) => sequence_number,
            None => {
                let chain_state_reader = RemoteStateReader::new(&self.client)?;
                chain_state_reader
                    .get_account_resource(*sender.address())?
                    .map(|account| account.sequence_number())
                    .ok_or_else(|| {
                        format_err!(
                            "Can not find account on chain by address:{}",
                            sender.address()
                        )
                    })?
            }
        };
        let node_info = self.client.node_info()?;
        let expiration_timestamp_secs = expiration_time_secs
            .unwrap_or(Self::DEFAULT_EXPIRATION_TIME_SECS)
            + node_info.now_seconds;
        Ok(RawUserTransaction::new(
            sender.address,
            sequence_number,
            payload,
            max_gas_amount.unwrap_or(Self::DEFAULT_MAX_GAS_AMOUNT),
            gas_price.unwrap_or(Self::DEFAULT_GAS_PRICE),
            expiration_timestamp_secs,
            chain_id,
            Self::DEFAULT_GAS_TOKEN.to_string(),
        ))
    }

    pub fn execute_transaction(
        &self,
        raw_txn: RawUserTransaction,
        only_dry_run: bool,
        blocking: bool,
    ) -> Result<ExecuteResultView> {
        let sender = self.get_account(raw_txn.sender())?;
        let (vm_status, output) = {
            let state_view = RemoteStateReader::new(&self.client)?;
            playground::dry_run(
                &state_view,
                DryRunTransaction {
                    public_key: sender.public_key,
                    raw_txn: raw_txn.clone(),
                },
            )?
        };
        if only_dry_run
            || !matches!(
                output.status(),
                TransactionStatus::Keep(KeptVMStatus::Executed)
            )
        {
            return Ok(ExecuteResultView::DryRun((vm_status, output.into())));
        }
        let signed_txn = self.client.account_sign_txn(raw_txn)?;

        let txn_hash = signed_txn.id();
        self.client.submit_transaction(signed_txn)?;
        eprintln!("txn {} submitted.", txn_hash);
        let mut output = ExecutionOutputView::new(txn_hash);
        if blocking {
            let (_block, txn_info) = self.watch_txn(txn_hash)?;
            output.txn_info = txn_info;
            let events = self.client.chain_get_events_by_txn_hash(txn_hash)?;
            output.events = Some(events);
        }
        Ok(ExecuteResultView::Run(output))
    }

    pub fn into_inner(self) -> (ChainNetworkID, Arc<RpcClient>, Option<NodeHandle>) {
        (self.net, self.client, self.node_handle)
    }
}
