// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod legacy;

use crate::define_storage;
use crate::storage::{CodecKVStore, ValueCodec};
use crate::{ContractEventStore, CONTRACT_EVENT_PREFIX_NAME_V2};
use anyhow::Result;
use bcs_ext::BCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_types::contract_event::StcContractEvent;
use starcoin_vm_types::contract_event::ContractEvent;

#[derive(Debug, Hash, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct StcContractEvents(Vec<StcContractEvent>);

define_storage!(
    StcContractEventStorage,
    HashValue,
    StcContractEvents,
    CONTRACT_EVENT_PREFIX_NAME_V2
);

impl From<Vec<ContractEvent>> for StcContractEvents {
    fn from(events: Vec<ContractEvent>) -> Self {
        Self(events.into_iter().map(Into::into).collect())
    }
}
impl From<Vec<StcContractEvent>> for StcContractEvents {
    fn from(events: Vec<StcContractEvent>) -> Self {
        Self(events)
    }
}
impl From<StcContractEvents> for Vec<StcContractEvent> {
    fn from(events: StcContractEvents) -> Self {
        events.0
    }
}

impl ValueCodec for StcContractEvents {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ContractEventStore for StcContractEventStorage {
    fn save_contract_events_v2(
        &self,
        txn_info_id: HashValue,
        events: Vec<StcContractEvent>,
    ) -> Result<()> {
        self.put(txn_info_id, events.into())
    }

    fn get_contract_events_v2(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<StcContractEvent>>> {
        Ok(self.get(txn_info_id)?.map(|events| events.into()))
    }

    fn save_contract_events(
        &self,
        txn_info_id: HashValue,
        events: Vec<ContractEvent>,
    ) -> Result<()> {
        self.save_contract_events_v2(
            txn_info_id,
            events.into_iter().map(Into::into).collect::<Vec<_>>(),
        )
    }

    fn get_contract_events(&self, txn_info_id: HashValue) -> Result<Option<Vec<ContractEvent>>> {
        let events = self.get_contract_events_v2(txn_info_id)?;
        Ok(events.map(|events| {
            events
                .into_iter()
                .filter_map(|e| e.to_v1())
                .collect::<Vec<_>>()
        }))
    }
}
