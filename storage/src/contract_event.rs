// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::define_storage;
use crate::storage::{CodecKVStore, ValueCodec};
use crate::{ContractEventStore, CONTRACT_EVENT_PREFIX_NAME, CONTRACT_EVENT_PREFIX_NAME_V2};
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_types::contract_event::{ContractEvent, StcContractEvent};

define_storage!(
    ContractEventStorage,
    HashValue,
    Vec<ContractEvent>,
    CONTRACT_EVENT_PREFIX_NAME
);

define_storage!(
    StcContractEventStorage,
    HashValue,
    Vec<StcContractEvent>,
    CONTRACT_EVENT_PREFIX_NAME_V2
);

impl ValueCodec for Vec<ContractEvent> {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec for Vec<StcContractEvent> {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

pub trait StcContractEventStore {
    fn save_contract_events_v2(
        &self,
        txn_info_id: HashValue,
        events: Vec<StcContractEvent>,
    ) -> Result<()>;

    fn get_contract_events_v2(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<StcContractEvent>>>;
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

    fn save_contract_events_v2(
        &self,
        _txn_info_id: HashValue,
        _events: Vec<StcContractEvent>,
    ) -> Result<()> {
        // This is a placeholder - the regular storage doesn't handle V2 events
        Ok(())
    }

    fn get_contract_events_v2(
        &self,
        _txn_info_id: HashValue,
    ) -> Result<Option<Vec<StcContractEvent>>> {
        // This is a placeholder - the regular storage doesn't handle V2 events
        Ok(None)
    }
}

impl StcContractEventStore for StcContractEventStorage {
    fn save_contract_events_v2(
        &self,
        txn_info_id: HashValue,
        events: Vec<StcContractEvent>,
    ) -> Result<()> {
        self.put(txn_info_id, events)
    }

    fn get_contract_events_v2(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<StcContractEvent>>> {
        self.get(txn_info_id)
    }
}
