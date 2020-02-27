// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{
    account_address::AccountAddress as LibraAccountAddress,
    byte_array::ByteArray as LibraByteArray,
    transaction::{
        Module as LibraModule, RawTransaction as LibraRawTransaction, Script as LibraScript,
        SignedTransaction as LibraSignedTransaction,
        TransactionArgument as LibraTransactionArgument,
        TransactionOutput as LibraTransactionOutput, TransactionPayload as LibraTransactionPayload,
    },
};
use types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    byte_array::ByteArray,
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

    pub fn to_libra_TransactionArgument(arg: &TransactionArgument) -> LibraTransactionArgument {
        match arg {
            TransactionArgument::U64(value) => LibraTransactionArgument::U64(*value),
            TransactionArgument::Bool(boolean) => LibraTransactionArgument::Bool(*boolean),
            TransactionArgument::Address(address) => {
                LibraTransactionArgument::Address(Self::to_libra_AccountAddress(*address))
            }
            TransactionArgument::ByteArray(byte_array) => LibraTransactionArgument::ByteArray(
                LibraByteArray::new((byte_array.clone()).into_inner()),
            ),
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
        }
    }
    pub fn to_starcoin_TransactionOutput(output: LibraTransactionOutput) -> TransactionOutput {
        TransactionOutput::new(
            vec![],
            0,
            TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
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
    LibraScript(Vec<u8>, Vec<LibraTransactionArgument>),
    LibraModule(Vec<u8>),
}
