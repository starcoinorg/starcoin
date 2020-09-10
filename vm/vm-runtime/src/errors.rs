// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_logger::prelude::*;
use starcoin_vm_types::account_config::ACCOUNT_MODULE;
use starcoin_vm_types::vm_status::{AbortLocation, StatusCode, VMStatus};

//should be consistent with ErrorCode.move
const PROLOGUE_ACCOUNT_DOES_NOT_EXIST: u64 = 0;
const PROLOGUE_INVALID_ACCOUNT_AUTH_KEY: u64 = 1;
const PROLOGUE_SEQUENCE_NUMBER_TOO_OLD: u64 = 2;
const PROLOGUE_SEQUENCE_NUMBER_TOO_NEW: u64 = 3;
const PROLOGUE_CANT_PAY_GAS_DEPOSIT: u64 = 4;
const PROLOGUE_TRANSACTION_EXPIRED: u64 = 5;
const PROLOGUE_BAD_CHAIN_ID: u64 = 6;

const EINSUFFICIENT_BALANCE: u64 = 10;
const ENOT_GENESIS_ACCOUNT: u64 = 11;
// Todo: haven't found proper StatusCode for below Error Code
const ENOT_GENESIS: u64 = 12;
const ECONFIG_VALUE_DOES_NOT_EXIST: u64 = 13;
const EINVALID_TIMESTAMP: u64 = 14;
const ECOIN_DEPOSIT_IS_ZERO: u64 = 15;
const EDESTORY_TOKEN_NON_ZERO: u64 = 16;
const EBLOCK_NUMBER_MISMATCH: u64 = 17;

pub fn convert_prologue_runtime_error(status: VMStatus) -> VMStatus {
    dbg!(status.clone());
    match status {
        VMStatus::MoveAbort(_location, code) => {
            let new_major_status = match code {
                PROLOGUE_ACCOUNT_DOES_NOT_EXIST => StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST,
                PROLOGUE_INVALID_ACCOUNT_AUTH_KEY => StatusCode::INVALID_AUTH_KEY,
                PROLOGUE_SEQUENCE_NUMBER_TOO_OLD => StatusCode::SEQUENCE_NUMBER_TOO_OLD,
                PROLOGUE_SEQUENCE_NUMBER_TOO_NEW => StatusCode::SEQUENCE_NUMBER_TOO_NEW,
                PROLOGUE_CANT_PAY_GAS_DEPOSIT => {
                    StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE
                }
                PROLOGUE_TRANSACTION_EXPIRED => StatusCode::TRANSACTION_EXPIRED,
                PROLOGUE_BAD_CHAIN_ID => StatusCode::BAD_CHAIN_ID,
                ENOT_GENESIS_ACCOUNT => StatusCode::NO_ACCOUNT_ROLE,
                ENOT_GENESIS => StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION,
                ECONFIG_VALUE_DOES_NOT_EXIST => {
                    StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION
                }
                EINVALID_TIMESTAMP => StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION,
                ECOIN_DEPOSIT_IS_ZERO => StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION,
                EDESTORY_TOKEN_NON_ZERO => StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION,
                EBLOCK_NUMBER_MISMATCH => StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION,
                // ToDo add corresponding error code into StatusCode
                _ => StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION,
            };
            VMStatus::Error(new_major_status)
        }
        status @ VMStatus::ExecutionFailure { .. } | status @ VMStatus::Executed => status,
        VMStatus::Error(code) => {
            error!("[starcoin_vm] Unexpected prologue error: {:?}", code);
            VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION)
        }
    }
}

pub fn convert_normal_success_epilogue_error(status: VMStatus) -> VMStatus {
    match status {
        VMStatus::MoveAbort(location, code @ EINSUFFICIENT_BALANCE) => {
            if location != account_module_abort() {
                error!(
                    "[starcoin_vm] Unexpected success epilogue move abort: {:?}::{:?}",
                    location, code
                );
                return VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION);
            }
            VMStatus::MoveAbort(location, code)
        }

        status @ VMStatus::Executed => status,

        status => {
            error!(
                "[starcoin_vm] Unexpected success epilogue error: {:?}",
                status
            );
            VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION)
        }
    }
}

fn account_module_abort() -> AbortLocation {
    AbortLocation::Module(ACCOUNT_MODULE.clone())
}
