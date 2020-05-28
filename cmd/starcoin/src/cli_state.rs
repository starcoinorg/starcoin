// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::{format_err, Result};
use starcoin_config::{ChainNetwork, DataDirPath};
use starcoin_crypto::HashValue;
use starcoin_node::NodeHandle;
use starcoin_rpc_client::RpcClient;
use starcoin_types::account_address::AccountAddress;
use starcoin_wallet_api::WalletAccount;
use std::path::Path;

pub struct CliState {
    net: ChainNetwork,
    client: RpcClient,
    join_handle: Option<NodeHandle>,
    temp_dir: DataDirPath,
}

impl CliState {
    pub fn new(net: ChainNetwork, client: RpcClient, join_handle: Option<NodeHandle>) -> CliState {
        Self {
            net,
            client,
            join_handle,
            temp_dir: starcoin_config::temp_path(),
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

    pub fn watch_txn(&self, txn_hash: HashValue) -> Result<()> {
        let block = self.client.watch_txn(txn_hash, None)?;
        println!(
            "txn mined in block hight: {}, hash: {:#x}",
            block.header().number(),
            block.header().id()
        );
        Ok(())
    }

    pub fn into_inner(self) -> (ChainNetwork, RpcClient, Option<NodeHandle>) {
        (self.net, self.client, self.join_handle)
    }
}
