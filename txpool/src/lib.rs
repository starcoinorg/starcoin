// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate trace_time;
extern crate transaction_pool as tx_pool;

use crate::txpool::TxPoolImpl;
use actix::prelude::*;
use anyhow::Result;
use bus::{Broadcast, BusActor, Subscription};

use traits::TxPoolAsyncService;
use types::{system_events::SystemEvents, transaction::SignedUserTransaction};

mod pool;
mod tx_pool_service_impl;
mod txpool;

pub use pool::TxStatus;
pub use tx_pool_service_impl::{CachedSeqNumberClient, SubscribeTxns, TxPool, TxPoolRef};

#[cfg(test)]
mod test;
