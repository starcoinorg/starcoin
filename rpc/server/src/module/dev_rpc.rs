// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use starcoin_dev::playground::PlaygroudService;
use starcoin_rpc_api::dev::DevApi;
use starcoin_rpc_api::FutureResult;
use starcoin_state_api::ChainStateAsyncService;
use starcoin_types::transaction::{SignedUserTransaction, Transaction, TransactionOutput};
use starcoin_vm_types::vm_status::VMStatus;

pub struct DevRpcImpl<S>
where
    S: ChainStateAsyncService + 'static,
{
    service: S,
    playground: PlaygroudService,
}

impl<S> DevRpcImpl<S>
where
    S: ChainStateAsyncService,
{
    pub fn new(service: S, playground: PlaygroudService) -> Self {
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
    fn dry_run(&self, txn: SignedUserTransaction) -> FutureResult<(VMStatus, TransactionOutput)> {
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
