// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{schema::contract_event::ContractEvent as ContractEventSchema, ContractEventStore};
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_schemadb::db::DBStorage;
use starcoin_types::contract_event::ContractEvent;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct ContractEventStorage {
    db: Arc<DBStorage>,
}

impl ContractEventStorage {
    pub(crate) fn new(db: &Arc<DBStorage>) -> Self {
        Self { db: Arc::clone(db) }
    }

    pub(crate) fn get(&self, key: &HashValue) -> Result<Option<Vec<ContractEvent>>> {
        self.db.get::<ContractEventSchema>(key)
    }
}

impl ContractEventStore for ContractEventStorage {
    fn save_contract_events(
        &self,
        txn_info_id: &HashValue,
        events: &Vec<ContractEvent>,
    ) -> Result<()> {
        self.db.put::<ContractEventSchema>(txn_info_id, events)
    }

    fn get_contract_events(&self, txn_info_id: &HashValue) -> Result<Option<Vec<ContractEvent>>> {
        self.db.get::<ContractEventSchema>(txn_info_id)
    }
}
