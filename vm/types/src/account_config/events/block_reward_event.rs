use crate::contract_event::ContractEvent;
use crate::language_storage::TypeTag;
use crate::move_resource::MoveResource;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockRewardEvent {
    pub block_number: u64,
    pub block_reward: u128,
    pub gas_fees: u128,
    pub miner: AccountAddress,
}

impl BlockRewardEvent {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for BlockRewardEvent {
    const MODULE_NAME: &'static str = "BlockReward";
    const STRUCT_NAME: &'static str = "BlockRewardEvent";
}

impl TryFrom<&ContractEvent> for BlockRewardEvent {
    type Error = anyhow::Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag() != &TypeTag::Struct(BlockRewardEvent::struct_tag()) {
            anyhow::bail!("Expected {}", Self::STRUCT_NAME);
        }
        Self::try_from_bytes(event.event_data())
    }
}
