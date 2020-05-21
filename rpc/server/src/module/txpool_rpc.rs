// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::future::TryFutureExt;
use starcoin_rpc_api::{txpool::TxPoolApi, FutureResult};
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::transaction::SignedUserTransaction;

/// Re-export the API
pub use starcoin_rpc_api::txpool::*;
use starcoin_types::account_address::AccountAddress;

pub struct TxPoolRpcImpl<S>
where
    S: TxPoolSyncService + 'static,
{
    service: S,
}

impl<S> TxPoolRpcImpl<S>
where
    S: TxPoolSyncService,
{
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

impl<S> TxPoolApi for TxPoolRpcImpl<S>
where
    S: TxPoolSyncService,
{
    fn submit_transaction(&self, txn: SignedUserTransaction) -> FutureResult<bool> {
        let result = self
            .service
            .add_txns(vec![txn])
            .pop()
            .expect("txpool should return result");
        let success = result.is_ok();
        Box::new(jsonrpc_core::futures::done(Ok(success)))
    }
    fn next_sequence_number(&self, address: AccountAddress) -> FutureResult<Option<u64>> {
        let result = self.service.next_sequence_number(address);
        Box::new(futures::future::ok(result).compat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonrpc_core::IoHandler;
    use starcoin_txpool_mock_service::MockTxPoolService;
    use tokio01::prelude::Future;

    #[test]
    fn test_submit_transaction() {
        let txn = SignedUserTransaction::mock();
        let result = serde_json::to_string(&txn).unwrap();
        println!("{}", result);
        let txn1 = serde_json::from_str::<SignedUserTransaction>(result.as_str()).unwrap();
        assert_eq!(txn, txn1);

        let mut io = IoHandler::new();
        let txpool_service = MockTxPoolService::new();
        io.extend_with(TxPoolRpcImpl::new(txpool_service).to_delegate());
        let request = r#"{"jsonrpc":"2.0","method":"txpool.submit_transaction","params":[{"public_key":"731fe437a8d3fbb25fa389307ac615e3a503e49be40e1b8cf9e5136fb44b9e5f","raw_txn":{"expiration_time":0,"gas_specifier":{"Struct":{"address":"00000000000000000000000000000000","module":"Starcoin","name":"T","type_params":[]}},"gas_unit_price":0,"max_gas_amount":0,"payload":{"Script":{"args":[],"ty_args":[],"code":[]}},"sender":"00000000000000000000000000000000","sequence_number":0},"signature":"6d2bcccb51de9046890e88e1a1c351b4b6342a1c59159074483ce511a17755ee778907ed6664ea637d7fabad1685de78cd277ca82ed8b75094e42901b152ef07"}],"id":0}"#;
        let response = r#"{"jsonrpc":"2.0","result":true,"id":0}"#;
        assert_eq!(
            io.handle_request(request).wait().unwrap(),
            Some(response.to_string())
        );
    }
}
