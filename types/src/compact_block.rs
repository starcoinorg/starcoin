use crate::block::{Block, BlockHeader, LegacyBlockHeader};
use crate::transaction::SignedUserTransaction;
use bcs_ext::Sample;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompactBlock {
    pub header: BlockHeader,
    pub short_ids: Vec<ShortId>,
    pub prefilled_txn: Vec<PrefilledTxn>,
    pub uncles: Option<Vec<BlockHeader>>,
    pub tips: Option<Vec<HashValue>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "CompactBlock")]
pub struct OldCompactBlock {
    pub header: LegacyBlockHeader,
    pub short_ids: Vec<ShortId>,
    pub prefilled_txn: Vec<PrefilledTxn>,
    pub uncles: Option<Vec<LegacyBlockHeader>>,
}

impl From<OldCompactBlock> for CompactBlock {
    fn from(value: OldCompactBlock) -> Self {
        Self {
            header: value.header.into(),
            short_ids: value.short_ids,
            prefilled_txn: value.prefilled_txn,
            uncles: value
                .uncles
                .map(|u| u.into_iter().map(Into::into).collect()),
            tips: None,
        }
    }
}

impl From<CompactBlock> for OldCompactBlock {
    fn from(value: CompactBlock) -> Self {
        Self {
            header: value.header.into(),
            short_ids: value.short_ids,
            prefilled_txn: value.prefilled_txn,
            uncles: value
                .uncles
                .map(|u| u.into_iter().map(Into::into).collect()),
        }
    }
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
    pub fn new(block: Block, tips: Option<Vec<HashValue>>) -> Self {
        let prefilled_txn: Vec<PrefilledTxn> = vec![];
        let header = block.header;
        let short_ids: Vec<ShortId> = block
            .body
            .transactions
            .into_iter()
            .map(|tx| tx.id())
            .map(ShortId)
            .collect();
        CompactBlock {
            header,
            short_ids,
            prefilled_txn,
            uncles: block.body.uncles,
            tips,
        }
    }

    pub fn txn_len(&self) -> usize {
        self.short_ids.len()
    }
}

impl From<Block> for CompactBlock {
    fn from(block: Block) -> Self {
        let tips = block
            .dag_parent_and_tips()
            .map(|(_, tips)| tips.iter().map(|b| b.id()).collect::<Vec<_>>());
        CompactBlock::new(block, tips)
    }
}

impl Sample for CompactBlock {
    fn sample() -> Self {
        // Block::sample().into()
        Self::new(Block::sample(), None)
    }
}
