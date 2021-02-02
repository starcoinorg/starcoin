// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::{convert_to_rpc_error, map_err};
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
/// Re-export the API
pub use starcoin_rpc_api::txpool::*;
use starcoin_rpc_api::types::{SignedUserTransactionView, StrView};
use starcoin_rpc_api::{txpool::TxPoolApi, FutureResult};
use starcoin_txpool_api::{TxPoolStatus, TxPoolSyncService};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::SignedUserTransaction;
use std::convert::TryInto;

/// Re-export the API
pub use starcoin_rpc_api::txpool::*;

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
    fn submit_transaction(&self, txn: SignedUserTransaction) -> FutureResult<HashValue> {
        let txn_hash = txn.id();
        let result: Result<(), jsonrpc_core::Error> = self
            .service
            .add_txns(vec![txn])
            .pop()
            .expect("txpool should return result")
            .map_err(convert_to_rpc_error);

        Box::pin(futures::future::ready(result.map(|_| txn_hash)))
    }

    fn submit_hex_transaction(&self, tx: String) -> FutureResult<HashValue> {
        let tx = tx.strip_prefix("0x").unwrap_or_else(|| tx.as_str());
        let result = hex::decode(tx)
            .map_err(convert_to_rpc_error)
            .and_then(|txn_bytes| SignedUserTransaction::decode(&txn_bytes).map_err(map_err))
            .and_then(|txn| {
                let txn_hash = txn.id();
                self.service
                    .add_txns(vec![txn])
                    .pop()
                    .expect("txpool should return result")
                    .map(|_| txn_hash)
                    .map_err(convert_to_rpc_error)
            });
        Box::pin(futures::future::ready(result))
    }

    fn gas_price(&self) -> FutureResult<StrView<u64>> {
        let gas_price = 1u64;
        Box::pin(futures::future::ok(gas_price.into()))
    }

    fn pending_txns(
        &self,
        addr: AccountAddress,
        max_len: Option<u32>,
    ) -> FutureResult<Vec<SignedUserTransactionView>> {
        let txns: Result<Vec<SignedUserTransactionView>, _> = self
            .service
            .txns_of_sender(&addr, max_len.map(|v| v as usize))
            .into_iter()
            .map(TryInto::try_into)
            .collect();
        Box::pin(futures::future::ready(txns.map_err(map_err)))
    }

    fn pending_txn(&self, txn_hash: HashValue) -> FutureResult<Option<SignedUserTransactionView>> {
        let txn = self
            .service
            .find_txn(&txn_hash)
            .map(TryInto::try_into)
            .transpose()
            .map_err(map_err);
        Box::pin(futures::future::ready(txn))
    }

    fn next_sequence_number(&self, address: AccountAddress) -> FutureResult<Option<u64>> {
        let result = self.service.next_sequence_number(address);
        Box::pin(futures::future::ok(result))
    }

    fn state(&self) -> FutureResult<TxPoolStatus> {
        let state = self.service.status();
        Box::pin(futures::future::ok(state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use jsonrpc_core::IoHandler;
    use starcoin_txpool_mock_service::MockTxPoolService;

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
        let txn_hash = txn.id();
        let prefix = r#"{"jsonrpc":"2.0","method":"txpool.submit_transaction","params":["#;
        let suffix = r#"],"id":0}"#;
        let request = format!(
            "{}{}{}",
            prefix,
            serde_json::to_string(&txn).expect("txn to json should success."),
            suffix
        );
        let response = r#"{"jsonrpc":"2.0","result":"$txn_hash","id":0}"#;
        let response = response.replace("$txn_hash", &txn_hash.to_string());

        assert_eq!(
            block_on(io.handle_request(request.as_str())).unwrap(),
            response
        );
    }
}
