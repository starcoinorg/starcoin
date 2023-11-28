// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    gas_algebra::NumBytes,
    vm_status::{KeptVMStatus, StatusCode, VMStatus},
};

use starcoin_language_e2e_tests::{
    common_transactions::peer_to_peer_txn, executor::FakeExecutor, test_with_different_versions,
    versioning::CURRENT_RELEASE_VERSIONS,
};

use starcoin_vm_runtime::{
    data_cache::{AsMoveResolver, StateViewCache},
    starcoin_vm::StarcoinVM,
};

use starcoin_vm_types::{
    account_config::G_STC_TOKEN_CODE,
    transaction_metadata::{TransactionMetadata, TransactionPayloadMetadata},
};

use starcoin_gas::StarcoinGasMeter;
use starcoin_gas_algebra_ext::{FeePerGasUnit, Gas};
use starcoin_vm_types::genesis_config::ChainId;

#[test]
fn failed_transaction_cleanup_test() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let balance = 1_000_000;
        let sender = executor.create_raw_account_data(balance, 10);
        executor.add_account_data(&sender);

        //let log_context = AdapterLogSchema::new(executor.get_state_view().id(), 0);
        let mut vm = StarcoinVM::new(None);
        let data_cache = StateViewCache::new(executor.get_state_view());
        vm.load_configs(&data_cache);

        let txn_data = TransactionMetadata {
            sender: *sender.address(),
            max_gas_amount: Gas::new(90_000),
            gas_unit_price: FeePerGasUnit::new(0),
            gas_token_code: G_STC_TOKEN_CODE.clone(),
            transaction_size: NumBytes::new(0),
            expiration_timestamp_secs: 0,
            sequence_number: 10,
            payload: TransactionPayloadMetadata::ScriptFunction,
            authentication_key_preimage: vec![],
            chain_id: ChainId::test(),
        };

        // let gas_schedule = zero_cost_schedule();
        // let mut gas_status = GasStatus::new(&gas_schedule, GasUnits::new(10_000));
        let gas_params = vm.get_gas_parameters().unwrap();
        let mut gas_meter = StarcoinGasMeter::new(gas_params.clone(), Gas::new(0 as u64));
        // TYPE_MISMATCH should be kept and charged.
        let (_, out1) = vm.failed_transaction_cleanup(
            VMStatus::Error(StatusCode::TYPE_MISMATCH),
            &mut gas_meter,
            &txn_data,
            &data_cache.as_move_resolver(),
        );
        assert!(!out1.write_set().is_empty());
        assert_eq!(out1.gas_used(), 90_000);
        assert!(!out1.status().is_discarded());
        assert_eq!(
            out1.status().status(),
            // StatusCode::TYPE_MISMATCH
            Ok(KeptVMStatus::MiscellaneousError)
        );

        // Invariant violations should be discarded and not charged.
        let (_, out2) = vm.failed_transaction_cleanup(
            VMStatus::Error(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR),
            &mut gas_meter,
            &txn_data,
            &data_cache.as_move_resolver(),
        );
        assert!(out2.write_set().is_empty());
        assert!(out2.gas_used() == 0);
        assert!(out2.status().is_discarded());
        assert_eq!(
            out2.status().status(),
            Err(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
        );
        }
    }
}

#[test]
fn non_existent_sender() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = FakeExecutor::from_test_genesis();
        let sequence_number = 0;
        let sender = executor.create_raw_account();
        let receiver = executor.create_raw_account_data(100_000, sequence_number);
        executor.add_account_data(&receiver);

        let transfer_amount = 10;
        let txn = peer_to_peer_txn(
            &sender,
            receiver.account(),
            sequence_number,
            transfer_amount,
        );

        let output = &executor.execute_transaction(txn);
        assert_eq!(
            output.status().status(),
            Err(StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST),
        );
    }
    }
}
