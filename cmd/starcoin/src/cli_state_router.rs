// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{ExecuteResultView, TransactionOptions};
use crate::view_vm2::ExecuteResultView as ExecuteResultViewVM2;
use crate::{cli_state_vm2::CliStateVM2, CliState};
use starcoin_account_api::{AccountInfo, AccountProvider};
use starcoin_config::ChainNetworkID;
use starcoin_node::NodeHandle;
use starcoin_rpc_client::RpcClient;
use std::path::PathBuf;

use anyhow::Result;
use starcoin_vm2_vm_types::transaction::TransactionPayload as TransactionPayloadV2;
use starcoin_vm_types::transaction::TransactionPayload;
use std::sync::Arc;
use std::time::Duration;

use starcoin_types::account_address::AccountAddress as AccountAddressV1;
use std::sync::atomic::{AtomicBool, Ordering};

static USING_VM2: AtomicBool = AtomicBool::new(false);

pub struct CliStateRouter {
    state_vm1: Option<CliState>,
    state_vm2: Option<CliStateVM2>,
    node_handle: Option<NodeHandle>,
}

impl CliStateRouter {
    pub fn new(
        using_vm2: bool,
        net: ChainNetworkID,
        client: Arc<RpcClient>,
        watch_timeout: Option<Duration>,
        node_handle: Option<NodeHandle>,
        account_client: Box<dyn AccountProvider>,
    ) -> Self {
        USING_VM2.store(using_vm2, Ordering::SeqCst);
        let (state_vm1, state_vm2) = if using_vm2 {
            (
                None,
                Some(CliStateVM2::new(net, client, watch_timeout, account_client)),
            )
        } else {
            (
                Some(CliState::new(net, client, watch_timeout, account_client)),
                None,
            )
        };
        CliStateRouter {
            state_vm1,
            state_vm2,
            node_handle,
        }
    }

    pub fn build_and_execute_transaction(
        &self,
        txn_opts: TransactionOptions,
        payload: TransactionPayload,
    ) -> Result<ExecuteResultView> {
        self.cli_state_vm1()
            .build_and_execute_transaction(txn_opts, payload)
    }

    pub fn build_and_execute_transaction_vm2(
        &self,
        txn_opts: TransactionOptions,
        payload: TransactionPayloadV2,
    ) -> Result<ExecuteResultViewVM2> {
        self.cli_state_vm2()
            .build_and_execute_transaction(txn_opts, payload)
    }

    fn cli_state_vm1(&self) -> &CliState {
        self.state_vm1
            .as_ref()
            .expect("vm1 is not initialized, please restart and select initialize to vm1")
    }

    fn cli_state_vm2(&self) -> &CliStateVM2 {
        self.state_vm2
            .as_ref()
            .expect("vm2 is not initialized, please restart and select initialize to vm2")
    }

    pub fn net(&self) -> &ChainNetworkID {
        if self.state_vm1.is_some() {
            return self.cli_state_vm1().net();
        }
        self.cli_state_vm2().net()
    }

    pub fn client(&self) -> &RpcClient {
        if self.state_vm1.is_some() {
            return self.cli_state_vm1().client();
        }
        self.cli_state_vm2().client()
    }

    pub fn account_client(&self) -> &dyn AccountProvider {
        if self.state_vm1.is_some() {
            return self.cli_state_vm1().account_client();
        }
        self.cli_state_vm2().account_client()
    }

    pub fn history_file(&self) -> PathBuf {
        if self.state_vm1.is_some() {
            return self.cli_state_vm1().history_file();
        }
        self.cli_state_vm2().history_file()
    }

    pub fn get_account_or_default(
        &self,
        account_address: Option<AccountAddressV1>,
    ) -> Result<AccountInfo> {
        self.cli_state_vm1().get_account_or_default(account_address)
    }

    pub fn into_inner(mut self) -> (ChainNetworkID, Arc<RpcClient>, Option<NodeHandle>) {
        let ret = if self.state_vm1.is_some() {
            self.state_vm1.take().unwrap().into_inner()
        } else {
            self.state_vm2.take().unwrap().into_inner()
        };
        (ret.0, ret.1, self.node_handle)
    }

    pub fn is_using_vm2() -> bool {
        USING_VM2.load(Ordering::SeqCst)
    }
}
