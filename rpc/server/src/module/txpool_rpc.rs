// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::{convert_to_rpc_error, map_err};
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
/// Re-export the API
use starcoin_rpc_api::{
    multi_types::MultiSignedUserTransactionView,
    types::{SignedUserTransactionView, StrView},
};
use starcoin_rpc_api::{txpool::TxPoolApi, FutureResult};
use starcoin_txpool_api::{TxPoolStatus, TxPoolSyncService};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::multi_transaction::{MultiAccountAddress, MultiSignedUserTransaction};
use starcoin_vm2_vm_types::account_address::AccountAddress as AccountAddress2;
use starcoin_vm2_vm_types::transaction::SignedUserTransaction as SignedUserTransaction2;
use starcoin_vm_types::transaction::SignedUserTransaction;
use std::convert::TryInto;

/// Re-export the API

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
        self.submit_transaction_multi(MultiSignedUserTransaction::VM1(txn))
    }

    fn submit_transaction2(&self, txn: SignedUserTransaction2) -> FutureResult<HashValue> {
        self.submit_transaction_multi(MultiSignedUserTransaction::VM2(txn))
    }

    fn submit_transaction_multi(&self, txn: MultiSignedUserTransaction) -> FutureResult<HashValue> {
        let txn_hash = txn.id();
        let result: Result<(), jsonrpc_core::Error> = self
            .service
            .add_txns_multi_signed(vec![txn])
            .pop()
            .expect("txpool should return result")
            .map_err(convert_to_rpc_error);

        Box::pin(futures::future::ready(result.map(|_| txn_hash)))
    }

    fn submit_hex_transaction(&self, tx: String) -> FutureResult<HashValue> {
        let tx = tx.strip_prefix("0x").unwrap_or(tx.as_str());
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

    fn submit_hex_transaction2(&self, tx: String) -> FutureResult<HashValue> {
        let tx = tx.strip_prefix("0x").unwrap_or(tx.as_str());
        let result = hex::decode(tx)
            .map_err(convert_to_rpc_error)
            .and_then(|txn_bytes| SignedUserTransaction2::decode(&txn_bytes).map_err(map_err))
            .and_then(|txn| {
                let txn_hash = txn.id();
                self.service
                    .add_txns_multi_signed(vec![MultiSignedUserTransaction::VM2(txn)])
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
        let multi_address = MultiAccountAddress::VM1(addr);
        let txns: Result<Vec<SignedUserTransactionView>, _> = self
            .service
            .txns_of_sender(&multi_address, max_len.map(|v| v as usize))
            .into_iter()
            .filter_map(|txn| match txn {
                MultiSignedUserTransaction::VM1(txn) => Some(txn),
                _ => None,
            })
            .map(TryInto::try_into)
            .collect();
        Box::pin(futures::future::ready(txns.map_err(map_err)))
    }

    fn pending_txns_multi(
        &self,
        addr: AccountAddress,
        max_len: Option<u32>,
    ) -> FutureResult<Vec<MultiSignedUserTransactionView>> {
        let multi_address = MultiAccountAddress::VM1(addr);
        let txns: Result<Vec<MultiSignedUserTransactionView>, _> = self
            .service
            .txns_of_sender(&multi_address, max_len.map(|v| v as usize))
            .into_iter()
            .map(TryInto::try_into)
            .collect();
        Box::pin(futures::future::ready(txns.map_err(map_err)))
    }

    fn pending_txn(&self, txn_hash: HashValue) -> FutureResult<Option<SignedUserTransactionView>> {
        let txn = self
            .service
            .find_txn(&txn_hash)
            .and_then(|txn| match txn {
                MultiSignedUserTransaction::VM1(t) => Some(t),
                _ => None,
            })
            .map(TryInto::try_into)
            .transpose()
            .map_err(map_err);

        Box::pin(futures::future::ready(txn))
    }

    fn pending_txn_multi(
        &self,
        txn_hash: HashValue,
    ) -> FutureResult<Option<MultiSignedUserTransactionView>> {
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

    fn next_sequence_number2(&self, address: AccountAddress2) -> FutureResult<Option<u64>> {
        let result = self.service.next_sequence_number2(address);
        Box::pin(futures::future::ok(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use jsonrpc_core::IoHandler;
    use starcoin_txpool_mock_service::MockTxPoolService;
    use starcoin_types::account::{peer_to_peer_txn, Account};
    use starcoin_vm2_types::{
        account::{peer_to_peer_txn as peer_to_peer_txn2, Account as Account2},
        transaction::SignedUserTransaction as SignedUserTransaction2,
    };
    use starcoin_vm_types::transaction::TransactionPayload;

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

    #[test]
    fn test_submit_hex_transaction_v1_to_v2_not_compatible() {
        let alice = Account::new();
        let bob = Account::new();
        let txn1 = peer_to_peer_txn(&alice, &bob, 0, 10_000, 5_000, 255.into());
        let payload = txn1.payload().clone();
        println!("payload1 {:?}", payload);

        let txn_bytes = bcs_ext::to_bytes(&txn1).unwrap();
        let txn1 = SignedUserTransaction2::decode(&txn_bytes);
        assert!(txn1.is_ok());
        let script_function = match payload {
            TransactionPayload::ScriptFunction(s) => s,
            _ => panic!(),
        };
        // payload1 ScriptFunction(ScriptFunction { module: ModuleId { address: 0x00000000000000000000000000000001, name: Identifier("TransferScripts") }, function: Identifier("peer_to_peer_v2"), ty_args: [Struct(StructTag { address: 0x00000000000000000000000000000001, module: Identifier("STC"), name: Identifier("STC"), type_params: [] })], args: [[248, 41, 114, 187, 41, 9, 54, 78, 201, 220, 218, 226, 116, 49, 145, 185], [16, 39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]] })
        assert_eq!(script_function.function().as_str(), "TransferScripts");

        let alice2 = Account2::new();
        let bob2 = Account2::new();
        let payload2 = peer_to_peer_txn2(&alice2, &bob2, 0, 10_000, 5_000, 255.into())
            .payload()
            .clone();

        //  payload2 EntryFunction(EntryFunction { module: ModuleId { address: 0x00000000000000000000000000000001, name: Identifier("transfer_scripts") }, function: Identifier("peer_to_peer_v2"), ty_args: [Struct(StructTag { address: 0x00000000000000000000000000000001, module: Identifier("starcoin_coin"), name: Identifier("STC"), type_args: [] })], args: [[49, 168, 188, 110, 65, 29, 84, 144, 62, 98, 92, 76, 111, 114, 234, 38], [16, 39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]] })

        println!("payload2 {:?}", payload2);
    }
}
