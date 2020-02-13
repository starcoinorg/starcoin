// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use actix::prelude::*;
use types::transaction::SignedUserTransaction;

#[derive(Message)]
#[rtype(result = "u64")]
pub struct GetCounterMessage {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopMessage {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastTransactionMessage {
    pub tx: SignedUserTransaction,
}

/// message from peer
#[derive(Message)]
#[rtype(result = "()")]
pub enum PeerMessage {
    UserTransaction(SignedUserTransaction),
}
