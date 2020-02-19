// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use actix::prelude::*;
use anyhow::Result;
use types::transaction::SignedUserTransaction;

#[derive(Message)]
#[rtype(result = "u64")]
pub struct GetCounterMessage {}

/// message from peer
#[derive(Message)]
#[rtype(result = "Result<()>")]
pub enum PeerMessage {
    UserTransaction(SignedUserTransaction),
}
