// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ReputationChange;
use anyhow::*;
use bcs_ext::{BCSCodec, Sample};
use serde::{Deserialize, Serialize};
use starcoin_service_registry::ServiceRequest;
use starcoin_types::block::BlockInfo;
use starcoin_types::cmpact_block::CompactBlock;
use starcoin_types::peer_info::{PeerId, PeerInfo};
use starcoin_types::startup_info::ChainInfo;
use starcoin_types::transaction::SignedUserTransaction;
use std::borrow::Cow;

pub const TXN_PROTOCOL_NAME: &str = "/starcoin/txn/1";
pub const BLOCK_PROTOCOL_NAME: &str = "/starcoin/block/1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TransactionsMessage {
    pub txns: Vec<SignedUserTransaction>,
}

impl TransactionsMessage {
    pub fn new(txns: Vec<SignedUserTransaction>) -> Self {
        Self { txns }
    }

    pub fn transactions(self) -> Vec<SignedUserTransaction> {
        self.txns
    }
}

impl Sample for TransactionsMessage {
    fn sample() -> Self {
        Self::new(vec![SignedUserTransaction::sample()])
    }
}

/// Message of sending or receive block notification to network
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct CompactBlockMessage {
    pub compact_block: CompactBlock,
    pub block_info: BlockInfo,
}

impl CompactBlockMessage {
    pub fn new(compact_block: CompactBlock, block_info: BlockInfo) -> Self {
        Self {
            compact_block,
            block_info,
        }
    }
}

impl Sample for CompactBlockMessage {
    fn sample() -> Self {
        Self::new(CompactBlock::sample(), BlockInfo::sample())
    }
}

/// Network notification protocol message, change this type, maybe break the network protocol compatibility.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotificationMessage {
    Transactions(TransactionsMessage),
    CompactBlock(Box<CompactBlockMessage>),
}

impl NotificationMessage {
    pub fn decode_notification(protocol_name: &str, bytes: &[u8]) -> Result<Self> {
        Ok(match protocol_name {
            TXN_PROTOCOL_NAME => {
                NotificationMessage::Transactions(TransactionsMessage::decode(bytes)?)
            }
            BLOCK_PROTOCOL_NAME => {
                NotificationMessage::CompactBlock(Box::new(CompactBlockMessage::decode(bytes)?))
            }
            unknown_protocol => bail!(
                "Unknown protocol {}'s message: {}",
                unknown_protocol,
                hex::encode(bytes)
            ),
        })
    }

    pub fn encode_notification(&self) -> Result<(Cow<'static, str>, Vec<u8>)> {
        Ok(match self {
            NotificationMessage::Transactions(msg) => (TXN_PROTOCOL_NAME.into(), msg.encode()?),
            NotificationMessage::CompactBlock(msg) => (BLOCK_PROTOCOL_NAME.into(), msg.encode()?),
        })
    }

    pub fn protocol_name(&self) -> Cow<'static, str> {
        match self {
            Self::Transactions(_) => TXN_PROTOCOL_NAME.into(),
            Self::CompactBlock(_) => BLOCK_PROTOCOL_NAME.into(),
        }
    }

    pub fn protocols() -> Vec<Cow<'static, str>> {
        vec![TXN_PROTOCOL_NAME.into(), BLOCK_PROTOCOL_NAME.into()]
    }

    pub fn into_transactions(self) -> Option<TransactionsMessage> {
        match self {
            NotificationMessage::Transactions(message) => Some(message),
            _ => None,
        }
    }

    pub fn into_compact_block(self) -> Option<CompactBlockMessage> {
        match self {
            NotificationMessage::CompactBlock(message) => Some(*message),
            _ => None,
        }
    }
}

/// Message for send or receive from peer
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeerMessage {
    pub peer_id: PeerId,
    pub notification: NotificationMessage,
}

impl PeerMessage {
    pub fn new(peer_id: PeerId, notification: NotificationMessage) -> Self {
        Self {
            peer_id,
            notification,
        }
    }
    pub fn new_transactions(peer_id: PeerId, transactions: TransactionsMessage) -> Self {
        Self::new(peer_id, NotificationMessage::Transactions(transactions))
    }

    pub fn new_compact_block(peer_id: PeerId, compact_block: CompactBlockMessage) -> Self {
        Self::new(
            peer_id,
            NotificationMessage::CompactBlock(Box::new(compact_block)),
        )
    }

    pub fn into_transactions(self) -> Option<PeerTransactionsMessage> {
        let peer_id = self.peer_id;
        self.notification
            .into_transactions()
            .map(|message| PeerTransactionsMessage { peer_id, message })
    }

    pub fn into_compact_block(self) -> Option<PeerCompactBlockMessage> {
        let peer_id = self.peer_id;
        self.notification
            .into_compact_block()
            .map(|message| PeerCompactBlockMessage { peer_id, message })
    }
}

impl ServiceRequest for PeerMessage {
    type Response = Result<()>;
}

/// Message for combine PeerId and TransactionsMessage
#[derive(Clone, Debug)]
pub struct PeerTransactionsMessage {
    pub peer_id: PeerId,
    pub message: TransactionsMessage,
}

impl PeerTransactionsMessage {
    pub fn new(peer_id: PeerId, message: TransactionsMessage) -> Self {
        Self { peer_id, message }
    }
}

impl Into<PeerMessage> for PeerTransactionsMessage {
    fn into(self) -> PeerMessage {
        PeerMessage::new_transactions(self.peer_id, self.message)
    }
}

/// Message for combine PeerId and CompactBlockMessage
#[derive(Clone, Debug)]
pub struct PeerCompactBlockMessage {
    pub peer_id: PeerId,
    pub message: CompactBlockMessage,
}

impl PeerCompactBlockMessage {
    pub fn new(peer_id: PeerId, message: CompactBlockMessage) -> Self {
        Self { peer_id, message }
    }
}

impl Into<PeerMessage> for PeerCompactBlockMessage {
    fn into(self) -> PeerMessage {
        PeerMessage::new_compact_block(self.peer_id, self.message)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PeerEvent {
    Open(PeerId, Box<ChainInfo>),
    Close(PeerId),
}

/// Network service message
#[derive(Clone, Debug)]
pub struct ReportReputation {
    pub peer_id: PeerId,
    pub change: ReputationChange,
}

#[derive(Clone, Debug)]
pub struct GetPeerSet;

impl ServiceRequest for GetPeerSet {
    type Response = Vec<PeerInfo>;
}

#[derive(Clone, Debug)]
pub struct GetPeerById {
    pub peer_id: PeerId,
}

impl ServiceRequest for GetPeerById {
    type Response = Option<PeerInfo>;
}

#[derive(Clone, Debug)]
pub struct GetSelfPeer;

impl ServiceRequest for GetSelfPeer {
    type Response = PeerInfo;
}
