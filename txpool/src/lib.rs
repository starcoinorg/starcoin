// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate trace_time;
extern crate transaction_pool as tx_pool;

use anyhow::{format_err, Result};
use network_api::messages::PeerTransactionsMessage;
pub use pool::queue::Pool;
pub use pool::scoring::SeqNumberAndGasPrice;
pub use pool::verifier::Verifier;
pub use pool::TxStatus;
pub use pool::{PoolTransaction, UnverifiedUserTransaction, VerifiedTransaction, VerifierOptions};
pub use pool_client::{NonceCache, PoolClient};
use starcoin_config::NodeConfig;
use starcoin_executor::VMMetrics;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_storage::Storage2;
use starcoin_storage::{BlockStore, Storage};
use starcoin_txpool_api::{PropagateTransactions, TxnStatusFullEvent};
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_types::{sync_status::SyncStatus, system_events::SyncStatusChangeEvent};
use starcoin_vm2_state_api::AccountStateReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tx_pool_service_impl::Inner;
pub use tx_pool_service_impl::TxPoolService;
pub use verifier_pool::VerifierPool;

mod metrics;
mod pending_transaction;
mod pool;
mod pool_client;
#[cfg(test)]
mod test;
mod tx_pool_actor_service;
mod tx_pool_service_impl;
mod verifier_pool;

pub use pool::queue::Pool;
pub use pool::TxStatus;
pub use tx_pool_actor_service::TxPoolActorService;
pub use tx_pool_service_impl::TxPoolService;
