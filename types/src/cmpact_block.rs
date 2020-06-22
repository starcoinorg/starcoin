use crate::block::{Block, BlockHeader};
use crate::transaction::{SignedUserTransaction, Transaction};
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompactBlock {
    pub header: BlockHeader,
    pub short_ids: Vec<ShortId>,
    pub prefilled_txn: Vec<PrefiledTxn>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PrefiledTxn {
    index: u64, //todo: change to a small one
    tx: SignedUserTransaction,
}

// TODO: change to siphash24 of 6bites
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShortId(HashValue);

impl CompactBlock {
    pub fn new(block: &Block, prefilled_txn: Vec<PrefiledTxn>) -> Self {
        let header = block.header.clone();
        let short_ids: Vec<ShortId> = block
            .transactions()
            .iter()
            .map(|tx| Transaction::UserTransaction(tx.clone()).id())
            .map(|id| ShortId(id))
            .collect();
        CompactBlock {
            header,
            short_ids,
            prefilled_txn,
        }
    }
}
