// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use starcoin_system::get_free_mem_size;
use std::sync::Arc;

pub const DEFAULT_MEM_SIZE: u64 = 128 * 1024 * 1024; // 128M

#[derive(Default, Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct TxPoolConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "txpool-max-count", long)]
    /// Maximal number of transactions in the pool. default to 4096
    max_count: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "txpool-max-per-sender", long)]
    /// Maximal number of transactions from single sender. default to 128
    max_per_sender: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "txpool-max-mem-usage", long)]
    /// Maximal memory usage. Default to half of current free mem of system.
    max_mem_usage: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "txpool-tx-propagate-interval", long)]
    /// interval(s) of tx propagation timer. default to 2.
    tx_propagate_interval: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "txpool-min-gas-price", long)]
    /// reject transaction whose gas_price is less than the min_gas_price. default to 1.
    min_gas_price: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "txpool-max-vm1-txn-count", long)]
    /// Max number of VM1 transactions allowed in the pool. default to 100.
    max_vm1_txn_count: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "txpool-max-vm1-rejections-per-peer", long)]
    /// Max number of VM1 rejections per peer before blacklisting. default to 10.
    max_vm1_rejections_per_peer: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "txpool-vm1-peer-blacklist-duration-secs", long)]
    /// Duration (in seconds) for which a peer remains blacklisted. default to 120.
    vm1_peer_blacklist_duration_secs: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "txpool-verifier-pool-size", long)]
    /// Size of the verifier pool used by txpool validation. default to num_cpus.
    verifier_pool_size: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(name = "txpool-verifier-pool-enabled", long)]
    /// Enable pooled VM validators for txpool verification. default to true.
    verifier_pool_enabled: Option<bool>,
}

impl TxPoolConfig {
    pub fn set_max_count(&mut self, max_count: u64) {
        self.max_count = Some(max_count);
    }
    pub fn set_max_per_sender(&mut self, max_per_sender: u64) {
        self.max_per_sender = Some(max_per_sender);
    }
    pub fn set_tx_propagate_interval(&mut self, interval: u64) {
        self.tx_propagate_interval = Some(interval);
    }
    pub fn max_count(&self) -> u64 {
        // Higher default to better support stress/bench workloads.
        self.max_count.unwrap_or(40_960_000)
    }
    pub fn max_per_sender(&self) -> u64 {
        self.max_per_sender.unwrap_or(40_960)
    }
    pub fn max_mem_usage(&self) -> u64 {
        self.max_mem_usage
            .unwrap_or_else(|| match get_free_mem_size() {
                Ok(free) => {
                    if free > 0 {
                        free / 2
                    } else {
                        DEFAULT_MEM_SIZE
                    }
                }
                Err(_) => DEFAULT_MEM_SIZE,
            })
    }
    pub fn tx_propagate_interval(&self) -> u64 {
        self.tx_propagate_interval.unwrap_or(2)
    }
    pub fn min_gas_price(&self) -> u64 {
        self.min_gas_price.unwrap_or(1)
    }
    pub fn max_vm1_txn_count(&self) -> usize {
        self.max_vm1_txn_count.unwrap_or(100)
    }
    pub fn max_vm1_rejections_per_peer(&self) -> usize {
        self.max_vm1_rejections_per_peer.unwrap_or(10)
    }
    pub fn vm1_peer_blacklist_duration_secs(&self) -> u64 {
        self.vm1_peer_blacklist_duration_secs.unwrap_or(120)
    }
    pub fn verifier_pool_size(&self) -> usize {
        self.verifier_pool_size.unwrap_or_else(num_cpus::get)
    }
    pub fn verifier_pool_enabled(&self) -> bool {
        self.verifier_pool_enabled.unwrap_or(true)
    }
}

impl ConfigModule for TxPoolConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, _base: Arc<BaseConfig>) -> Result<()> {
        let txpool_opt = &opt.txpool;
        if let Some(m) = txpool_opt.max_mem_usage.as_ref() {
            self.max_mem_usage = Some(*m);
        }
        if let Some(m) = txpool_opt.max_count.as_ref() {
            self.max_count = Some(*m);
        }
        if let Some(m) = txpool_opt.max_mem_usage.as_ref() {
            self.max_mem_usage = Some(*m);
        }
        if let Some(m) = txpool_opt.tx_propagate_interval.as_ref() {
            self.tx_propagate_interval = Some(*m);
        }
        if let Some(m) = txpool_opt.min_gas_price.as_ref() {
            self.min_gas_price = Some(*m);
        }
        if let Some(m) = txpool_opt.max_vm1_txn_count.as_ref() {
            self.max_vm1_txn_count = Some(*m);
        }
        if let Some(m) = txpool_opt.max_vm1_rejections_per_peer.as_ref() {
            self.max_vm1_rejections_per_peer = Some(*m);
        }
        if let Some(m) = txpool_opt.vm1_peer_blacklist_duration_secs.as_ref() {
            self.vm1_peer_blacklist_duration_secs = Some(*m);
        }
        if let Some(m) = txpool_opt.verifier_pool_size.as_ref() {
            self.verifier_pool_size = Some(*m);
        }
        if let Some(m) = txpool_opt.verifier_pool_enabled.as_ref() {
            self.verifier_pool_enabled = Some(*m);
        }
        Ok(())
    }
}
