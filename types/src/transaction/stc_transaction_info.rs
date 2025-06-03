// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction::legacy::RichTransactionInfo;
use crate::transaction::TransactionInfo;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_vm2_types::transaction::{
    RichTransactionInfo as RichTransactionInfoV2, TransactionInfo as TransactionInfoV2,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StcTransactionInfo {
    V1(TransactionInfo),
    V2(TransactionInfoV2),
}

impl StcTransactionInfo {
    pub fn to_v1(self) -> Option<TransactionInfo> {
        match self {
            Self::V1(info) => Some(info),
            Self::V2(_) => None,
        }
    }

    pub fn to_v2(self) -> Option<TransactionInfoV2> {
        match self {
            Self::V1(_) => None,
            Self::V2(info) => Some(info),
        }
    }
}

impl From<TransactionInfo> for StcTransactionInfo {
    fn from(info: TransactionInfo) -> Self {
        Self::V1(info)
    }
}

impl From<TransactionInfoV2> for StcTransactionInfo {
    fn from(info: TransactionInfoV2) -> Self {
        Self::V2(info)
    }
}

/// `StcRichTransactionInfo` is a wrapper of `StcTransactionInfo` with more info,
/// such as `block_id`, `block_number` which is the block that include the txn producing the txn info.
/// We cannot put the block_id into txn_info, because txn_info is accumulated into block header.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StcRichTransactionInfo {
    pub block_id: HashValue,
    pub block_number: u64,
    pub transaction_info: StcTransactionInfo,
    /// Transaction index in block
    pub transaction_index: u32,
    /// Transaction global index in chain, equivalent to transaction accumulator's leaf index
    pub transaction_global_index: u64,
}

impl StcRichTransactionInfo {
    pub fn new(
        block_id: HashValue,
        block_number: u64,
        transaction_info: StcTransactionInfo,
        transaction_index: u32,
        transaction_global_index: u64,
    ) -> Self {
        Self {
            block_id,
            block_number,
            transaction_info,
            transaction_index,
            transaction_global_index,
        }
    }

    pub fn block_id(&self) -> HashValue {
        self.block_id
    }

    pub fn txn_info(&self) -> &StcTransactionInfo {
        &self.transaction_info
    }
}

impl From<RichTransactionInfo> for StcRichTransactionInfo {
    fn from(info: RichTransactionInfo) -> Self {
        Self {
            block_id: info.block_id,
            block_number: info.block_number,
            transaction_info: info.transaction_info.into(),
            transaction_index: info.transaction_index,
            transaction_global_index: info.transaction_global_index,
        }
    }
}

impl StcRichTransactionInfo {
    pub fn id(&self) -> HashValue {
        match &self.transaction_info {
            StcTransactionInfo::V1(info) => info.id(),
            StcTransactionInfo::V2(info) => info.id(),
        }
    }
    pub fn transaction_hash(&self) -> HashValue {
        match &self.transaction_info {
            StcTransactionInfo::V1(info) => info.transaction_hash(),
            StcTransactionInfo::V2(info) => info.transaction_hash(),
        }
    }
    pub fn to_v1(self) -> Option<RichTransactionInfo> {
        match self.transaction_info {
            StcTransactionInfo::V1(info) => Some(RichTransactionInfo {
                block_id: self.block_id,
                block_number: self.block_number,
                transaction_info: info,
                transaction_index: self.transaction_index,
                transaction_global_index: self.transaction_global_index,
            }),
            StcTransactionInfo::V2(_) => None,
        }
    }
    pub fn to_v2(self) -> Option<RichTransactionInfoV2> {
        match self.transaction_info {
            StcTransactionInfo::V1(_) => None,
            StcTransactionInfo::V2(info) => Some(RichTransactionInfoV2 {
                block_id: self.block_id,
                block_number: self.block_number,
                transaction_info: info,
                transaction_index: self.transaction_index,
                transaction_global_index: self.transaction_global_index,
            }),
        }
    }
}
