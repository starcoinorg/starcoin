// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{
    account_address::AccountAddress as LibraAccountAddress,
    contract_event::ContractEvent as LibraContractEvent,
    transaction::{
        Module as LibraModule, RawTransaction as LibraRawTransaction, Script as LibraScript,
        SignedTransaction as LibraSignedTransaction,
        TransactionArgument as LibraTransactionArgument,
        TransactionOutput as LibraTransactionOutput, TransactionPayload as LibraTransactionPayload,
        TransactionStatus as LibraTransactionStatus,
    },
    vm_error::{StatusCode as LibraStatusCode, VMStatus as LibraVMStatus},
};

use std::convert::TryFrom;
use types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    byte_array::ByteArray,
    contract_event::ContractEvent,
    transaction::{
        Module, RawUserTransaction, Script, SignedUserTransaction, TransactionArgument,
        TransactionOutput, TransactionPayload, TransactionStatus,
    },
    vm_error::{StatusCode, VMStatus},
    write_set::WriteSet,
};

pub struct TransactionHelper {}
impl TransactionHelper {
    pub fn to_libra_SignedTransaction(txn: &SignedUserTransaction) -> LibraSignedTransaction {
        let raw_txn = LibraRawTransaction::new(
            Self::to_libra_AccountAddress(txn.sender()),
            txn.sequence_number(),
            Self::to_libra_TransactionPayload(txn.payload().clone()),
            txn.max_gas_amount(),
            txn.gas_unit_price(),
            txn.expiration_time(),
        );
        LibraSignedTransaction::new(raw_txn, txn.public_key(), txn.signature())
    }
    pub fn to_libra_AccountAddress(address: AccountAddress) -> LibraAccountAddress {
        LibraAccountAddress::new(address.into_inner())
    }
    //    pub fn to_starcoin_AccountAddress(address: LibraAccountAddress) -> AccountAddress {
    //        let inner: [u8; ADDRESS_LENGTH] = *address.to_vec().as_bytes().try_into().unwrap();
    //        AccountAddress::new(inner)
    //    }

    pub fn to_libra_TransactionArgument(arg: &TransactionArgument) -> LibraTransactionArgument {
        match arg {
            TransactionArgument::U64(value) => LibraTransactionArgument::U64(*value),
            TransactionArgument::Bool(boolean) => LibraTransactionArgument::Bool(*boolean),
            TransactionArgument::Address(address) => {
                LibraTransactionArgument::Address(Self::to_libra_AccountAddress(*address))
            }
            TransactionArgument::U8Vector(_) => todo!(),
        }
    }
    pub fn to_libra_Script(s: Script) -> LibraScript {
        let args = s
            .args()
            .iter()
            .map(|arg| Self::to_libra_TransactionArgument(arg))
            .collect();
        LibraScript::new(s.code().to_vec(), args)
    }
    pub fn to_libra_Module(m: Module) -> LibraModule {
        LibraModule::new(m.code().to_vec())
    }
    pub fn to_libra_TransactionPayload(payload: TransactionPayload) -> LibraTransactionPayload {
        match payload {
            TransactionPayload::Script(s) => {
                LibraTransactionPayload::Script(Self::to_libra_Script(s))
            }
            TransactionPayload::Module(m) => {
                LibraTransactionPayload::Module(Self::to_libra_Module(m))
            }
            TransactionPayload::StateSet(_) => {
                unimplemented!("MockExecutor does not support StateSet transaction payload.")
            }
        }
    }
    pub fn to_starcoin_Events(_events: Vec<LibraContractEvent>) -> Vec<ContractEvent> {
        // ToDo: support ContractEvent
        vec![]
    }
    pub fn to_starcoin_VMStatus(status: LibraVMStatus) -> VMStatus {
        let major: u64 = status.major_status.into();
        VMStatus {
            major_status: StatusCode::try_from(major).unwrap(),
            sub_status: status.sub_status,
            message: status.message,
        }
    }
    pub fn to_libra_VMStatus(status: VMStatus) -> LibraVMStatus {
        let major: u64 = status.major_status.into();
        LibraVMStatus {
            major_status: LibraStatusCode::try_from(major).unwrap(),
            sub_status: status.sub_status,
            message: status.message,
        }
    }
    pub fn to_starcoin_TransactionStatus(status: &LibraTransactionStatus) -> TransactionStatus {
        match status {
            LibraTransactionStatus::Discard(vm_status) => {
                TransactionStatus::Discard(Self::to_starcoin_VMStatus(vm_status.clone()))
            }
            LibraTransactionStatus::Keep(vm_status) => {
                TransactionStatus::Keep(Self::to_starcoin_VMStatus(vm_status.clone()))
            }
            LibraTransactionStatus::Retry => {
                TransactionStatus::Discard(VMStatus::new(StatusCode::UNKNOWN_VALIDATION_STATUS))
            }
        }
    }
    pub fn to_starcoin_TransactionOutput(output: LibraTransactionOutput) -> TransactionOutput {
        TransactionOutput::new(
            Self::to_starcoin_Events(output.events().to_vec()),
            output.gas_used(),
            Self::to_starcoin_TransactionStatus(output.status()),
        )
    }
    pub fn fake_starcoin_TransactionOutput() -> TransactionOutput {
        TransactionOutput::new(
            vec![],
            0,
            TransactionStatus::Discard(VMStatus::new(StatusCode::ABORTED)),
        )
    }
}

pub enum VerifiedTranscationPayload {
    Script(Vec<u8>, Vec<TransactionArgument>),
    Module(Vec<u8>),
}
