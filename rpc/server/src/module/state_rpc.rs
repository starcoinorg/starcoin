// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::TryFutureExt;
use scs::SCSCodec;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::state::StateApi;
use starcoin_rpc_api::FutureResult;
use starcoin_state_api::{ChainStateAsyncService, StateWithProof};
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};

pub struct StateRpcImpl<S>
where
    S: ChainStateAsyncService + 'static,
{
    service: S,
}

impl<S> StateRpcImpl<S>
where
    S: ChainStateAsyncService,
{
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

impl<S> StateApi for StateRpcImpl<S>
where
    S: ChainStateAsyncService,
{
    fn get(&self, access_path: AccessPath) -> FutureResult<Option<Vec<u8>>> {
        let fut = self.service.clone().get(access_path).map_err(map_err);
        Box::new(fut.compat())
    }

    fn get_hex(&self, access_path_hex: String) -> FutureResult<Option<Vec<u8>>> {
        let access_path_bytes = match hex::decode(access_path_hex) {
            Ok(t) => t,
            Err(e) => return Box::new(jsonrpc_core::futures::failed(map_err(e.into()))),
        };
        let access_path = match AccessPath::decode(&access_path_bytes) {
            Ok(t) => t,
            Err(e) => return Box::new(jsonrpc_core::futures::failed(map_err(e))),
        };
        let fut = self.service.clone().get(access_path).map_err(map_err);
        Box::new(fut.compat())
    }

    fn get_with_proof(&self, access_path: AccessPath) -> FutureResult<StateWithProof> {
        let fut = self
            .service
            .clone()
            .get_with_proof(access_path)
            .map_err(map_err);
        Box::new(fut.compat())
    }

    fn get_account_state(&self, address: AccountAddress) -> FutureResult<Option<AccountState>> {
        let fut = self
            .service
            .clone()
            .get_account_state(address)
            .map_err(map_err);
        Box::new(fut.compat())
    }

    fn get_state_root(&self) -> FutureResult<HashValue> {
        let fut = self.service.clone().state_root().map_err(map_err);
        Box::new(fut.compat())
    }
}
