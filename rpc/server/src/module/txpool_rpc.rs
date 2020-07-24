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
    fn submit_transaction(&self, txn: SignedUserTransaction) -> FutureResult<Result<(), String>> {
        let result = self
            .service
            .add_txns(vec![txn])
            .pop()
            .expect("txpool should return result");
        Box::new(jsonrpc_core::futures::done(Ok(
            result.map_err(|e| format!("{:?}", e))
        )))
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
        let txn1 = serde_json::from_str::<SignedUserTransaction>(result.as_str()).unwrap();
        assert_eq!(txn, txn1);

        let mut io = IoHandler::new();
        let txpool_service = MockTxPoolService::new();
        io.extend_with(TxPoolRpcImpl::new(txpool_service).to_delegate());
        let txn = SignedUserTransaction::mock();
        let prefix = r#"{"jsonrpc":"2.0","method":"txpool.submit_transaction","params":["#;
        let suffix = r#"],"id":0}"#;
        let request = format!(
            "{}{}{}",
            prefix,
            serde_json::to_string(&txn).expect("txn to json should success."),
            suffix
        );
        let response = r#"{"jsonrpc":"2.0","result":{"Ok":null},"id":0}"#;
        assert_eq!(
            io.handle_request(request.as_str()).wait().unwrap(),
            Some(response.to_string())
        );
    }
}
