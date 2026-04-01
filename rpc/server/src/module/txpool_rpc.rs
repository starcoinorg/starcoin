// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::{convert_to_rpc_error, map_err};
use bcs_ext::BCSCodec;
use jsonrpsee::core::{async_trait, RpcResult};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::txpool::TxPoolApiServer;
/// Re-export the API
use starcoin_rpc_api::{
    multi_types::MultiSignedUserTransactionView,
    types::{SignedUserTransactionView, StrView},
};
use starcoin_txpool_api::{TxPoolStatus, TxPoolSyncService};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::multi_transaction::{
    MultiAccountAddress, MultiSignedUserTransaction, MultiTransactionError,
};
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

    fn submit_multi_signed_transactions(
        &self,
        txns: Vec<MultiSignedUserTransaction>,
        bypass_vm1_limit: bool,
        local_peer_id: Option<String>,
    ) -> RpcResult<Vec<HashValue>> {
        let txn_hashes = txns.iter().map(|txn| txn.id()).collect::<Vec<_>>();
        let results = self
            .service
            .add_txns_multi_signed(txns, bypass_vm1_limit, local_peer_id)
            .map_err(convert_to_rpc_error)
            .map_err(crate::module::map_jsonrpc_err)?;
        self.ensure_submission_results(&txn_hashes, results)?;
        Ok(txn_hashes)
    }

    fn submit_transaction_multi(&self, txn: MultiSignedUserTransaction) -> RpcResult<HashValue> {
        let bypass_vm1_limit = matches!(txn, MultiSignedUserTransaction::VM2(_));
        let local_peer_id = (!bypass_vm1_limit).then(|| "local-rpc".to_string());
        self.submit_multi_signed_transactions(vec![txn], bypass_vm1_limit, local_peer_id)
            .map(|mut txn_hashes| txn_hashes.pop().expect("single txn must yield one hash"))
    }

    fn ensure_submission_results(
        &self,
        txn_hashes: &[HashValue],
        results: Vec<Result<(), MultiTransactionError>>,
    ) -> RpcResult<()> {
        if results.len() != txn_hashes.len() {
            return Err(crate::module::map_jsonrpc_err(anyhow::anyhow!(
                "txpool returned {} results for {} transactions",
                results.len(),
                txn_hashes.len()
            )));
        }

        for result in results {
            result
                .map_err(convert_to_rpc_error)
                .map_err(crate::module::map_jsonrpc_err)?;
        }

        Ok(())
    }
}

#[async_trait]
impl<S> TxPoolApiServer for TxPoolRpcImpl<S>
where
    S: TxPoolSyncService,
{
    async fn submit_transaction(&self, txn: SignedUserTransaction) -> RpcResult<HashValue> {
        self.submit_transaction_multi(MultiSignedUserTransaction::VM1(txn))
    }

    async fn submit_transaction2(&self, txn: SignedUserTransaction2) -> RpcResult<HashValue> {
        self.submit_transaction_multi(MultiSignedUserTransaction::VM2(txn))
    }

    async fn submit_transactions(
        &self,
        txns: Vec<SignedUserTransaction>,
    ) -> RpcResult<Vec<HashValue>> {
        let txns = txns.into_iter().map(Into::into).collect();
        self.submit_multi_signed_transactions(txns, false, Some("local-rpc".to_string()))
    }

    async fn submit_hex_transaction(&self, tx: String) -> RpcResult<HashValue> {
        let tx = tx.strip_prefix("0x").unwrap_or(tx.as_str());
        let txn = hex::decode(tx)
            .map_err(convert_to_rpc_error)
            .and_then(|txn_bytes| SignedUserTransaction::decode(&txn_bytes).map_err(map_err))
            .map_err(crate::module::map_jsonrpc_err)?;
        self.submit_transaction_multi(MultiSignedUserTransaction::VM1(txn))
    }

    async fn submit_hex_transaction2(&self, tx: String) -> RpcResult<HashValue> {
        let tx = tx.strip_prefix("0x").unwrap_or(tx.as_str());
        let txn = hex::decode(tx)
            .map_err(convert_to_rpc_error)
            .and_then(|txn_bytes| SignedUserTransaction2::decode(&txn_bytes).map_err(map_err))
            .map_err(crate::module::map_jsonrpc_err)?;
        self.submit_transaction_multi(MultiSignedUserTransaction::VM2(txn))
    }

    async fn gas_price(&self) -> RpcResult<StrView<u64>> {
        let gas_price = 1u64;
        Ok(gas_price.into())
    }

    async fn pending_txns(
        &self,
        addr: AccountAddress,
        max_len: Option<u32>,
    ) -> RpcResult<Vec<SignedUserTransactionView>> {
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
        txns.map_err(map_err)
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn pending_txns_multi(
        &self,
        addr: AccountAddress,
        max_len: Option<u32>,
    ) -> RpcResult<Vec<MultiSignedUserTransactionView>> {
        let multi_address = MultiAccountAddress::VM1(addr);
        let txns: Result<Vec<MultiSignedUserTransactionView>, _> = self
            .service
            .txns_of_sender(&multi_address, max_len.map(|v| v as usize))
            .into_iter()
            .map(TryInto::try_into)
            .collect();
        txns.map_err(map_err)
            .map_err(crate::module::map_jsonrpc_err)
    }

    async fn pending_txn(
        &self,
        txn_hash: HashValue,
    ) -> RpcResult<Option<SignedUserTransactionView>> {
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

        txn.map_err(crate::module::map_jsonrpc_err)
    }

    async fn pending_txn_multi(
        &self,
        txn_hash: HashValue,
    ) -> RpcResult<Option<MultiSignedUserTransactionView>> {
        let txn = self
            .service
            .find_txn(&txn_hash)
            .map(TryInto::try_into)
            .transpose()
            .map_err(map_err);
        txn.map_err(crate::module::map_jsonrpc_err)
    }

    async fn next_sequence_number(&self, address: AccountAddress) -> RpcResult<Option<u64>> {
        let result = self.service.next_sequence_number(address);
        Ok(result)
    }

    async fn next_sequence_number_in_batch(
        &self,
        addresses: Vec<AccountAddress>,
    ) -> RpcResult<Option<Vec<(AccountAddress, Option<u64>)>>> {
        let result = self.service.next_sequence_number_in_batch(addresses);
        Ok(result)
    }

    async fn state(&self) -> RpcResult<TxPoolStatus> {
        let state = self.service.status();
        Ok(state)
    }

    async fn next_sequence_number2(&self, address: AccountAddress2) -> RpcResult<Option<u64>> {
        let result = self.service.next_sequence_number2(address);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use futures_channel::mpsc;
    use starcoin_rpc_api::txpool::txpool_methods;
    use starcoin_txpool_api::TxnStatusFullEvent;
    use starcoin_txpool_mock_service::MockTxPoolService;
    use starcoin_types::account::{peer_to_peer_txn, Account};
    use starcoin_types::block::Block;
    use starcoin_types::multi_transaction::{MultiAccountAddress, MultiTransactionError};
    use starcoin_types::transaction::TransactionError;
    use starcoin_vm2_types::{
        account::{peer_to_peer_txn as peer_to_peer_txn2, Account as Account2},
        transaction::SignedUserTransaction as SignedUserTransaction2,
    };
    use starcoin_vm_types::transaction::TransactionPayload;
    use std::sync::Arc;

    #[derive(Clone)]
    struct FailingTxPoolService {
        results: Vec<Result<(), MultiTransactionError>>,
    }

    impl FailingTxPoolService {
        fn new(results: Vec<Result<(), MultiTransactionError>>) -> Self {
            Self { results }
        }
    }

    impl TxPoolSyncService for FailingTxPoolService {
        fn add_txns_multi_signed(
            &self,
            _txns: Vec<MultiSignedUserTransaction>,
            _bypass_vm1_limit: bool,
            _peer_id: Option<String>,
        ) -> anyhow::Result<Vec<Result<(), MultiTransactionError>>> {
            Ok(self.results.clone())
        }

        fn remove_txn(
            &self,
            _txn_hash: HashValue,
            _is_invalid: bool,
        ) -> Option<MultiSignedUserTransaction> {
            unimplemented!()
        }

        fn get_pending_txns(
            &self,
            _max_len: Option<u64>,
            _now: Option<u64>,
        ) -> anyhow::Result<Vec<MultiSignedUserTransaction>> {
            unimplemented!()
        }

        fn get_pending_with_state(
            &self,
            _max_len: u64,
            _current_timestamp_secs: Option<u64>,
            _state_root1: HashValue,
            _state_root2: HashValue,
        ) -> anyhow::Result<Vec<MultiSignedUserTransaction>> {
            unimplemented!()
        }

        fn next_sequence_number(&self, _address: AccountAddress) -> Option<u64> {
            unimplemented!()
        }

        fn next_sequence_number_in_batch(
            &self,
            _addresses: Vec<AccountAddress>,
        ) -> Option<Vec<(AccountAddress, Option<u64>)>> {
            unimplemented!()
        }

        fn subscribe_txns(&self) -> mpsc::UnboundedReceiver<TxnStatusFullEvent> {
            unimplemented!()
        }

        fn subscribe_pending_txn(&self) -> mpsc::UnboundedReceiver<Arc<[HashValue]>> {
            unimplemented!()
        }

        fn chain_new_block(
            &self,
            _enacted: Vec<Block>,
            _retracted: Vec<Block>,
        ) -> anyhow::Result<()> {
            unimplemented!()
        }

        fn status(&self) -> TxPoolStatus {
            unimplemented!()
        }

        fn find_txn(&self, _hash: &HashValue) -> Option<MultiSignedUserTransaction> {
            unimplemented!()
        }

        fn txns_of_sender(
            &self,
            _sender: &MultiAccountAddress,
            _max_len: Option<usize>,
        ) -> Vec<MultiSignedUserTransaction> {
            unimplemented!()
        }

        fn next_sequence_number2(&self, _address: AccountAddress2) -> Option<u64> {
            unimplemented!()
        }

        fn next_sequence_number2_in_batch(
            &self,
            _addresses: Vec<AccountAddress2>,
        ) -> Option<Vec<(AccountAddress2, Option<u64>)>> {
            unimplemented!()
        }
    }

    #[test]
    fn test_submit_transaction() {
        let txn = SignedUserTransaction::mock();
        let result = serde_json::to_string(&txn).unwrap();
        let txn1 = serde_json::from_str::<SignedUserTransaction>(result.as_str()).unwrap();
        assert_eq!(txn, txn1);

        let txpool_service = MockTxPoolService::new();
        let methods =
            txpool_methods(TxPoolRpcImpl::new(txpool_service)).expect("register txpool methods");
        let txn = SignedUserTransaction::mock();
        let txn_hash = txn.id();
        let response: HashValue = block_on(methods.call("txpool.submit_transaction", [txn]))
            .expect("submit transaction via rpc should success");

        assert_eq!(response, txn_hash);
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
            _ => panic!(
                "Unexpected TransactionPayload variant encountered; expected ScriptFunction."
            ),
        };
        // payload1 ScriptFunction(ScriptFunction { module: ModuleId { address: 0x00000000000000000000000000000001, name: Identifier("TransferScripts") }, function: Identifier("peer_to_peer_v2"), ty_args: [Struct(StructTag { address: 0x00000000000000000000000000000001, module: Identifier("STC"), name: Identifier("STC"), type_params: [] })], args: [[248, 41, 114, 187, 41, 9, 54, 78, 201, 220, 218, 226, 116, 49, 145, 185], [16, 39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]] })
        assert_eq!(script_function.module().name().as_str(), "TransferScripts");

        let alice2 = Account2::new();
        let bob2 = Account2::new();
        let payload2 = peer_to_peer_txn2(&alice2, &bob2, 0, 10_000, 5_000, 255.into())
            .payload()
            .clone();

        //  payload2 EntryFunction(EntryFunction { module: ModuleId { address: 0x00000000000000000000000000000001, name: Identifier("transfer_scripts") }, function: Identifier("peer_to_peer_v2"), ty_args: [Struct(StructTag { address: 0x00000000000000000000000000000001, module: Identifier("starcoin_coin"), name: Identifier("STC"), type_args: [] })], args: [[49, 168, 188, 110, 65, 29, 84, 144, 62, 98, 92, 76, 111, 114, 234, 38], [16, 39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]] })

        println!("payload2 {:?}", payload2);
    }

    #[test]
    fn test_submit_transactions_returns_first_failure() {
        let txpool_service = FailingTxPoolService::new(vec![
            Err(MultiTransactionError::VM1(TransactionError::Old)),
            Ok(()),
        ]);
        let txpool = TxPoolRpcImpl::new(txpool_service);
        let err = block_on(txpool.submit_transactions(vec![
            SignedUserTransaction::mock(),
            SignedUserTransaction::mock(),
        ]))
        .expect_err("batch submission should fail when any txn is rejected");

        assert_eq!(err.code(), jsonrpsee::types::error::INVALID_PARAMS_CODE);
        assert!(err.message().contains("No longer valid"));
    }
}
