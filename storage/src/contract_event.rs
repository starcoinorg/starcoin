// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::define_storage;
use crate::storage::{CodecKVStore, ValueCodec};
use crate::{ContractEventStore, CONTRACT_EVENT_PREFIX_NAME};
use anyhow::Result;
use bcs_ext::BCSCodec;
use crypto::HashValue;
use starcoin_types::contract_event::ContractEvent;

define_storage!(
    ContractEventStorage,
    HashValue,
    Vec<ContractEvent>,
    CONTRACT_EVENT_PREFIX_NAME
);

impl ValueCodec for Vec<ContractEvent> {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ContractEventStore for ContractEventStorage {
    fn save_contract_events(
        &self,
        txn_info_id: HashValue,
        events: Vec<ContractEvent>,
    ) -> Result<()> {
        self.put(txn_info_id, events)
    }

    fn get_contract_events(&self, txn_info_id: HashValue) -> Result<Option<Vec<ContractEvent>>> {
        self.get(txn_info_id)
    }
}
