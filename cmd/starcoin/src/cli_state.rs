// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::{format_err, Result};
use starcoin_config::{ChainNetwork, DataDirPath};
use starcoin_crypto::HashValue;
use starcoin_node::NodeHandle;
use starcoin_rpc_api::types::pubsub::ThinBlock;
use starcoin_rpc_client::RpcClient;
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::association_address;
use starcoin_wallet_api::WalletAccount;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio_compat::runtime::Runtime;

static HISTORY_FILE_NAME: &str = "history";

pub struct CliState {
    net: ChainNetwork,
    client: Arc<RpcClient>,
    join_handle: Option<NodeHandle>,
    /// Cli data dir, different with Node data dir.
    data_dir: PathBuf,
    temp_dir: DataDirPath,
    _rt: Option<Runtime>,
}

impl CliState {
    pub const DEFAULT_WATCH_TIMEOUT: Duration = Duration::from_secs(300);
    pub fn new(
        net: ChainNetwork,
        client: Arc<RpcClient>,
        join_handle: Option<NodeHandle>,
        rt: Option<Runtime>,
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
            join_handle,
            data_dir,
            temp_dir,
            _rt: rt,
        }
    }

    pub fn net(&self) -> ChainNetwork {
        self.net
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

    pub fn default_account(&self) -> Result<WalletAccount> {
        self.client
            .wallet_default()?
            .ok_or_else(|| format_err!("Can not find default account, Please input from account."))
    }

    pub fn wallet_account_or_default(
        &self,
        account_address: Option<AccountAddress>,
    ) -> Result<WalletAccount> {
        if let Some(account_address) = account_address {
            self.client.wallet_get(account_address)?.ok_or_else(|| {
                format_err!("Can not find WalletAccount by address: {}", account_address)
            })
        } else {
            self.default_account()
        }
    }

    pub fn association_account(&self) -> Result<Option<WalletAccount>> {
        self.client.wallet_get(association_address())
    }

    pub fn watch_txn(&self, txn_hash: HashValue) -> Result<ThinBlock> {
        let block = self
            .client
            .watch_txn(txn_hash, Some(Self::DEFAULT_WATCH_TIMEOUT))?;
        println!(
            "txn mined in block height: {}, hash: {:#x}",
            block.header().number(),
            block.header().id()
        );
        Ok(block)
    }

    pub fn into_inner(self) -> (ChainNetwork, Arc<RpcClient>, Option<NodeHandle>) {
        (self.net, self.client, self.join_handle)
    }
}
