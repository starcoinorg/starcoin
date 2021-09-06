use crate::block::{Block, BlockHeader};
use crate::transaction::{SignedUserTransaction, Transaction};
use bcs_ext::Sample;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompactBlock {
    pub header: BlockHeader,
    pub short_ids: Vec<ShortId>,
    pub prefilled_txn: Vec<PrefilledTxn>,
    pub uncles: Option<Vec<BlockHeader>>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PrefilledTxn {
    pub index: u64,
    pub tx: SignedUserTransaction,
}

// TODO: change to siphash24 of 6bites
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShortId(pub HashValue);

impl CompactBlock {
    pub fn new(block: Block, prefilled_txn: Vec<PrefilledTxn>) -> Self {
        let header = block.header;
        let short_ids: Vec<ShortId> = block
            .body
            .transactions
            .into_iter()
            .map(|tx| Transaction::UserTransaction(tx).id())
            .map(ShortId)
            .collect();
        CompactBlock {
            header,
            short_ids,
            prefilled_txn,
            uncles: block.body.uncles,
        }
    }

    pub fn txn_len(&self) -> usize {
        self.short_ids
            .len()
            .saturating_add(self.prefilled_txn.len())
    }
}

impl From<Block> for CompactBlock {
    fn from(block: Block) -> Self {
        CompactBlock::new(block, vec![])
    }
}

impl Sample for CompactBlock {
    fn sample() -> Self {
        Block::sample().into()
    }
}
