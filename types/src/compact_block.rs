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
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename = "CompactBlock")]
pub struct LegacyCompactBlock {
    pub header: LegacyBlockHeader,
    pub short_ids: Vec<ShortId>,
    pub prefilled_txn: Vec<PrefilledTxn>,
    pub uncles: Option<Vec<LegacyBlockHeader>>,
}

impl From<LegacyCompactBlock> for CompactBlock {
    fn from(value: LegacyCompactBlock) -> Self {
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

impl From<CompactBlock> for LegacyCompactBlock {
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
    pub fn new(block: Block) -> Self {
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
        }
    }

    pub fn txn_len(&self) -> usize {
        self.short_ids.len()
    }
}

impl From<Block> for CompactBlock {
    fn from(block: Block) -> Self {
        CompactBlock::new(block)
    }
}

impl Sample for CompactBlock {
    fn sample() -> Self {
        Block::sample().into()
    }
}

#[cfg(test)]
mod tests {
    use super::{CompactBlock, LegacyCompactBlock, ShortId};
    use crate::block::BlockHeader;
    use bcs_ext::BCSCodec;
    use starcoin_crypto::HashValue;

    fn setup_data() -> (LegacyCompactBlock, CompactBlock) {
        let header = BlockHeader::random();
        let uncles = vec![BlockHeader::random(), BlockHeader::random()];
        let short_ids = vec![ShortId(HashValue::random()), ShortId(HashValue::random())];
        let legacy = LegacyCompactBlock {
            header: header.clone().into(),
            short_ids: short_ids.clone(),
            prefilled_txn: vec![],
            uncles: Some(uncles.iter().cloned().map(Into::into).collect()),
        };

        let block = CompactBlock {
            header,
            short_ids,
            prefilled_txn: vec![],
            uncles: Some(uncles),
        };
        (legacy, block)
    }

    #[test]
    fn test_compact_block_converting() {
        let (legacy, block) = setup_data();

        let converted_block: CompactBlock = legacy.clone().into();
        assert_eq!(block, converted_block);

        let converted_legacy: LegacyCompactBlock = block.into();
        assert_eq!(legacy, converted_legacy);
    }

    #[test]
    fn test_compact_block_encode_decode() {
        let (legacy, block) = setup_data();

        // legacy format -> upgraded format
        let legacy_raw = legacy.encode().unwrap();
        let de_legacy = LegacyCompactBlock::decode(&legacy_raw).unwrap();
        assert_eq!(legacy, de_legacy);
        assert!(CompactBlock::decode(&legacy_raw).is_err());
        let converted_block: CompactBlock = de_legacy.into();
        assert_eq!(block, converted_block);

        // upgraded format -> legacy format
        let converted_legacy: LegacyCompactBlock = block.into();
        let converted_legacy_raw = converted_legacy.encode().unwrap();
        assert_eq!(legacy_raw, converted_legacy_raw);
    }
}
