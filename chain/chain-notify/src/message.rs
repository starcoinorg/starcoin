// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_types::block::BlockHeader;
use starcoin_types::{block::BlockNumber, contract_event::ContractEvent};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Notification<T>(pub T);

impl<T> actix::Message for Notification<T> {
    type Result = ();
}

pub type ContractEventNotification = Notification<Arc<Vec<Event>>>;
pub type NewHeadEventNotification = Notification<ThinBlock>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub block_hash: HashValue,
    pub block_number: BlockNumber,
    pub transaction_hash: HashValue,
    // txn index in block
    pub transaction_index: Option<u64>,
    pub contract_event: ContractEvent,
}

impl Event {
    pub fn new(
        block_hash: HashValue,
        block_number: BlockNumber,
        transaction_hash: HashValue,
        transaction_index: Option<u64>,
        contract_event: ContractEvent,
    ) -> Self {
        Self {
            block_hash,
            block_number,
            transaction_hash,
            transaction_index,
            contract_event,
        }
    }
}

/// Block with only txn hashes.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ThinBlock {
    pub header: BlockHeader,
    pub body: Vec<HashValue>,
}
impl ThinBlock {
    pub fn new(header: BlockHeader, txn_hashes: Vec<HashValue>) -> Self {
        Self {
            header,
            body: txn_hashes,
        }
    }
    pub fn header(&self) -> &BlockHeader {
        &self.header
    }
    pub fn body(&self) -> &[HashValue] {
        &self.body
    }
}
