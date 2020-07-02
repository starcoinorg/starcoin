// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use anyhow::Result;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_crypto::HashValue;
use starcoin_executor::execute_transactions;
use starcoin_rpc_api::dev::DevApi;
use starcoin_rpc_api::FutureResult;
use starcoin_state_api::{ChainStateAsyncService, StateNodeStore};
use starcoin_statedb::ChainStateDB;
use starcoin_types::transaction::{SignedUserTransaction, Transaction, TransactionOutput};
use std::sync::Arc;

pub struct DevRpcImpl<S>
where
    S: ChainStateAsyncService + 'static,
{
    service: S,
    playground: DevPlaygroudService,
}

impl<S> DevRpcImpl<S>
where
    S: ChainStateAsyncService,
{
    pub fn new(service: S, playground: DevPlaygroudService) -> Self {
        Self {
            service,
            playground,
        }
    }
}

impl<S> DevApi for DevRpcImpl<S>
where
    S: ChainStateAsyncService,
{
    fn dry_run(&self, txn: SignedUserTransaction) -> FutureResult<TransactionOutput> {
        let service = self.service.clone();
        let playground = self.playground.clone();
        let f = async move {
            let state_root = service.state_root().await?;
            let output = playground.dry_run(state_root, Transaction::UserTransaction(txn))?;
            Ok(output)
        }
        .map_err(map_err);
        Box::new(f.boxed().compat())
    }
}

#[derive(Clone)]
pub struct DevPlaygroudService {
    pub state: Arc<dyn StateNodeStore>,
}

impl DevPlaygroudService {
    pub fn new(state_store: Arc<dyn StateNodeStore>) -> Self {
        Self { state: state_store }
    }
}

impl DevPlaygroudService {
    pub fn dry_run(&self, state_root: HashValue, txn: Transaction) -> Result<TransactionOutput> {
        let state_view = ChainStateDB::new(self.state.clone(), Some(state_root));
        execute_transactions(&state_view, vec![txn]).map(|mut r| r.pop().unwrap())
    }
}
