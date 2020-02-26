use crate::block::BlockNumber;
use starcoin_crypto::hash::HashValue;
/// Uniquely identifies block.
#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
pub enum BlockId {
    /// Block's sha3.
    /// Querying by hash is always faster.
    Hash(HashValue),
    /// Block number within canon blockchain.
    Number(BlockNumber),
    /// Earliest block (genesis).
    Earliest,
    /// Latest mined block.
    Latest,
}

/// Uniquely identifies transaction.
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum TransactionId {
    /// Transaction's sha3.
    Hash(HashValue),
    /// Block id and transaction index within this block.
    /// Querying by block position is always faster.
    Location(BlockId, usize),
}

// /// Uniquely identifies Trace.
// pub struct TraceId {
//     /// Transaction
//     pub transaction: TransactionId,
//     /// Trace address within transaction.
//     pub address: Vec<usize>,
// }

/// Uniquely identifies Uncle.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct UncleId {
    /// Block id.
    pub block: BlockId,
    /// Position in block.
    pub position: usize,
}
