use crate::account_address::AccountAddress;
use crate::contract_event::ContractEvent;
use crate::language_storage::TypeTag;
use crate::move_resource::MoveResource;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct NewBlockEvent {
    number: u64,
    author: AccountAddress,
    timestamp: u64,
    uncles: u64,
}
impl NewBlockEvent {
    pub fn try_from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        bcs_ext::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for NewBlockEvent {
    const MODULE_NAME: &'static str = "Block";
    const STRUCT_NAME: &'static str = "NewBlockEvent";
}

impl TryFrom<&ContractEvent> for NewBlockEvent {
    type Error = anyhow::Error;

    fn try_from(event: &ContractEvent) -> anyhow::Result<Self> {
        if event.type_tag() != &TypeTag::Struct(Self::struct_tag()) {
            anyhow::bail!("Expected {}", Self::STRUCT_NAME);
        }
        Self::try_from_bytes(event.event_data())
    }
}
