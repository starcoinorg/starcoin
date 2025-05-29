use crate::block::BlockNumber;
use crate::contract_event::{ContractEvent, StcContractEvent};
use starcoin_crypto::HashValue;
use starcoin_vm2_types::contract_event::ContractEventInfo as ContractEventInfo2;

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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct StcContractEventInfo {
    pub block_hash: HashValue,
    pub block_number: BlockNumber,
    pub transaction_hash: HashValue,
    /// txn index in block
    pub transaction_index: u32,
    /// txn global index in chain
    pub transaction_global_index: u64,
    /// event index in the transaction events.
    pub event_index: u32,
    pub event: StcContractEvent,
}

impl TryFrom<StcContractEventInfo> for ContractEventInfo {
    type Error = anyhow::Error;

    fn try_from(value: StcContractEventInfo) -> Result<Self, Self::Error> {
        let StcContractEventInfo {
            block_hash,
            block_number,
            transaction_hash,
            transaction_index,
            transaction_global_index,
            event_index,
            event,
        } = value;
        match event {
            StcContractEvent::V1(event) => Ok(ContractEventInfo {
                block_hash,
                block_number,
                transaction_hash,
                transaction_index,
                transaction_global_index,
                event_index,
                event,
            }),
            StcContractEvent::V2(_event) => Err(anyhow::anyhow!(
                "StcContractEvent V2 is not compatible with V1"
            )),
        }
    }
}

impl TryFrom<StcContractEventInfo> for ContractEventInfo2 {
    type Error = anyhow::Error;

    fn try_from(value: StcContractEventInfo) -> Result<Self, Self::Error> {
        let StcContractEventInfo {
            block_hash,
            block_number,
            transaction_hash,
            transaction_index,
            transaction_global_index,
            event_index,
            event,
        } = value;
        match event {
            StcContractEvent::V2(event) => Ok(ContractEventInfo2 {
                block_hash,
                block_number,
                transaction_hash,
                transaction_index,
                transaction_global_index,
                event_index,
                event,
            }),
            StcContractEvent::V1(_event) => Err(anyhow::anyhow!(
                "StcContractEvent V2 is not compatible with V1"
            )),
        }
    }
}
