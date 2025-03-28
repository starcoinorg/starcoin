// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![deny(clippy::arithmetic_side_effects)]

pub mod helper;
mod network_metrics;
pub mod network_p2p_handle;
mod service;
pub mod service_ref;
pub mod worker;

pub use network_api::messages::*;

pub use helper::{get_unix_ts, get_unix_ts_as_millis};
pub use network_p2p_types::peer_id::PeerId;
pub use service::NetworkActorService;
pub use service_ref::NetworkServiceRef;
pub use worker::build_network_worker;
