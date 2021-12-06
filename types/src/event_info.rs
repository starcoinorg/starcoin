use crate::block::BlockNumber;
use crate::contract_event::ContractEvent;
use starcoin_crypto::HashValue;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ContractEventInfo {
    pub block_hash: HashValue,
    pub block_number: BlockNumber,
    pub transaction_hash: HashValue,
    /// txn index in block
    pub transaction_index: u32,
    /// txn global index in chain
    pub transaction_global_index: u64,
    /// event index in the transaction events.
    pub event_index: u32,
    pub event: ContractEvent,
}
