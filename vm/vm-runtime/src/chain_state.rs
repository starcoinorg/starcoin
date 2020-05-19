// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use crypto::HashValue;
use libra_state_view::StateView;
use move_vm_state::data_cache::RemoteCache;
use starcoin_logger::prelude::*;
use starcoin_state_api::ChainState;
use starcoin_vm_types::{
    access_path::AccessPath,
    write_set::{WriteOp, WriteSet},
};
use vm::errors::VMResult;
//TODO this adaptor may be remove?
/// Adaptor for chain state
pub struct StateStore<'txn> {
    chain_state: &'txn dyn ChainState,
}

impl<'txn> StateStore<'txn> {
    pub fn new(chain_state: &'txn dyn ChainState) -> Self {
        StateStore { chain_state }
    }

    /// Adds a [`WriteSet`] to state store.
    pub fn add_write_set(&mut self, write_set: &WriteSet) {
        for (access_path, write_op) in write_set {
            match write_op {
                WriteOp::Value(blob) => {
                    self.set(access_path.clone(), blob.clone())
                        .unwrap_or_else(|e| panic!("Failure to set access path: {}", e));
                }
                WriteOp::Deletion => {
                    self.remove(access_path)
                        .unwrap_or_else(|e| panic!("Failure to remove access path: {}", e));
                }
            }
        }
    }

    /// Sets a (key, value) pair within state store.
    pub fn set(&mut self, access_path: AccessPath, data_blob: Vec<u8>) -> Result<()> {
        self.chain_state.set(&access_path, data_blob)
    }

    /// Deletes a key from state store.
    pub fn remove(&mut self, access_path: &AccessPath) -> Result<()> {
        self.chain_state.remove(access_path)
    }

    #[allow(dead_code)]
    pub fn state(&mut self) -> &'txn dyn ChainState {
        self.chain_state
    }

    #[allow(dead_code)]
    pub fn commit(&self) -> Result<HashValue> {
        self.chain_state.commit()
    }

    #[allow(dead_code)]
    pub fn flush(&self) -> Result<()> {
        self.chain_state.flush()
    }
}

/// read-only snapshot of the global state, to construct remote cache
impl<'txn> StateView for StateStore<'txn> {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        let result = self.chain_state.get(access_path);
        match result {
            Ok(remote_data) => Ok(remote_data),
            Err(e) => {
                error!("fail to read access_path, err: {:?}", e);
                Err(e)
            }
        }
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!();
    }

    fn is_genesis(&self) -> bool {
        unimplemented!();
    }
}

/// data cache, to construct transaction execution context
impl<'txn> RemoteCache for StateStore<'txn> {
    fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        Ok(StateView::get(self, access_path).expect("it should not error"))
    }
}
