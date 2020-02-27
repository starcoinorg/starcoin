// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path_helper::AccessPathHelper;
use anyhow::{Error, Result};
use libra_state_view::StateView;
use libra_types::access_path::AccessPath;
use std::sync::Arc;
use traits::ChainState;
use types::access_path::AccessPath as StarcoinAccessPath;
use vm::errors::VMResult;
use vm_runtime::data_cache::{BlockDataCache, RemoteCache};
/// Adaptor for chain state

pub struct StateStore<'txn> {
    chain_state: &'txn dyn ChainState,
}

impl<'txn> StateStore<'txn> {
    pub fn new(chain_state: &'txn dyn ChainState) -> Self {
        StateStore { chain_state }
    }
}

impl<'txn> StateView for StateStore<'txn> {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        ChainState::get(
            self.chain_state,
            &AccessPathHelper::to_Starcoin_AccessPath(access_path),
        )
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!();
    }

    fn is_genesis(&self) -> bool {
        unimplemented!();
    }
}

// This is used by the `process_transaction` API.
impl<'txn> RemoteCache for StateStore<'txn> {
    fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        Ok(StateView::get(self, access_path).expect("it should not error"))
    }
}
