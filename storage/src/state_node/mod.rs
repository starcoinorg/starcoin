// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod mock;

use crate::schema::state_node::State;
use anyhow::Result;
use starcoin_schemadb::{db::DBStorage, schema::Schema, SchemaBatch};
use std::{marker::PhantomData, sync::Arc};

pub use mock::StateStorageMock;

pub(crate) type StateStorage = StateStore<State>;
#[derive(Clone)]
pub(crate) struct StateStore<S: Schema> {
    db: Arc<DBStorage>,
    _phantom: PhantomData<S>,
}

impl<S: Schema> StateStore<S> {
    pub(crate) fn new(db: &Arc<DBStorage>) -> Self {
        Self {
            db: Arc::clone(db),
            _phantom: Default::default(),
        }
    }

    pub(crate) fn get(&self, key: &S::Key) -> Result<Option<S::Value>> {
        self.db.get::<S>(key)
    }

    pub(crate) fn put(&self, key: &S::Key, value: &S::Value) -> Result<()> {
        self.db.put::<S>(key, value)
    }

    pub(crate) fn write_schemas(&self, batch: SchemaBatch) -> Result<()> {
        self.db.write_schemas(batch)
    }
}
