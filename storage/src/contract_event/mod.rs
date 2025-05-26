// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod legacy;

use crate::define_storage;
use crate::storage::{CodecKVStore, ValueCodec};
use crate::{ContractEventStore, CONTRACT_EVENT_PREFIX_NAME_V2};
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_types::contract_event::StcContractEvent;
use starcoin_vm_types::contract_event::ContractEvent;

define_storage!(
    StcContractEventStorage,
    HashValue,
    Vec<StcContractEvent>,
    CONTRACT_EVENT_PREFIX_NAME_V2
);

impl ValueCodec for Vec<StcContractEvent> {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ContractEventStore for StcContractEventStorage {
    fn save_contract_events_v2(&self, txn_info_id: HashValue, events: Vec<StcContractEvent>) -> Result<()> {
        self.put(txn_info_id, events)
    }

    fn get_contract_events_v2(&self, txn_info_id: HashValue) -> Result<Option<Vec<StcContractEvent>>> {
        self.get(txn_info_id)
    }

    fn save_contract_events(
        &self,
        txn_info_id: HashValue,
        events: Vec<ContractEvent>,
    ) -> Result<()> {
        self.put(txn_info_id, events.into_iter().map(Into::into).collect::<Vec<_>>())
    }

    fn get_contract_events(&self, txn_info_id: HashValue) -> Result<Option<Vec<ContractEvent>>> {
        let events = self.get(txn_info_id)?;
        Ok(
            events.map(|events| events.into_iter().filter_map(|e| e.to_v1()).collect::<Vec<_>>()),
        )
    }
}
