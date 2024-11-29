// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_logger::prelude::*;
use starcoin_vm_types::account_config::G_ACCOUNT_MODULE;
use starcoin_vm_types::errors::VMError;
use starcoin_vm_types::vm_status::{AbortLocation, StatusCode, VMStatus};

//should be consistent with move code
// see stc_transaction_validation.move
const EPROLOGUE_ACCOUNT_DOES_NOT_EXIST: u64 = 1000;
const EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY: u64 = 1001;
const EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD: u64 = 1002;
const EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW: u64 = 1003;
const EPROLOGUE_CANT_PAY_GAS_DEPOSIT: u64 = 1004;
const EPROLOGUE_TRANSACTION_EXPIRED: u64 = 1005;
const EPROLOGUE_BAD_CHAIN_ID: u64 = 1006;
const EPROLOGUE_MODULE_NOT_ALLOWED: u64 = 1007;
const EPROLOGUE_SCRIPT_NOT_ALLOWED: u64 = 1008;
const EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG: u64 = 1009;
const EINSUFFICIENT_BALANCE: u64 = 1010;

// see chain_status.move
const ENOT_GENESIS: u64 = 1012;

// see on_chain_config.move
const ECONFIG_VALUE_DOES_NOT_EXIST: u64 = 1013;

// see timestamp.move
const EINVALID_TIMESTAMP: u64 = 1014;

// see stc_transaction_validation.move
const ECOIN_DEPOSIT_IS_ZERO: u64 = 1015;

// see stc_block.move
const EBLOCK_NUMBER_MISMATCH: u64 = 1017;

// see stc_transaction_validation.move
const EBAD_TRANSACTION_FEE_TOKEN: u64 = 1018;
#[allow(unused)]
const EDEPRECATED_FUNCTION: u64 = 1019;

// see stc_transaction_validation.move
const EPROLOGUE_SIGNER_ALREADY_DELEGATED: u64 = 1200;

// todo: how to handle these error code
//const EPROLOGUE_SENDING_ACCOUNT_FROZEN: u64 = 10;
//const EPROLOGUE_SENDING_TXN_GLOBAL_FROZEN: u64 = 11;
//const ENOT_GENESIS_ACCOUNT: u64 = 11;

// todo: where?
const EDESTROY_TOKEN_NON_ZERO: u64 = 16;

// see error categories defined in Move-Stdlib `error.move`
const INVALID_ARGUMENT: u64 = 0x1;
const OUT_OF_RANGE: u64 = 0x2;
const INVALID_STATE: u64 = 0x3;
#[allow(unused)]
const UNAUTHENTICATED: u64 = 0x4;
#[allow(unused)]
const PERMISSION_DENIED: u64 = 0x5;
#[allow(unused)]
const NOT_FOUND: u64 = 0x6;
#[allow(unused)]
const ABORTED: u64 = 0x7;
const ALREADY_EXISTS: u64 = 0x8;
#[allow(unused)]
const RESOURCE_EXHAUSTED: u64 = 0x9;
#[allow(unused)]
const CANCELLED: u64 = 0xA;
#[allow(unused)]
const INTERNAL: u64 = 0xB;
#[allow(unused)]
const NOT_IMPLEMENTED: u64 = 0xC;
#[allow(unused)]
const UNAVAILABLE: u64 = 0xD;

// see `canonical` function defined in Move-Stdlib `error.move`
pub fn error_split(code: u64) -> (u64, u64) {
    let category = code >> 16;
    let reason = code - (category << 16);
    (category, reason)
}

pub fn convert_prologue_runtime_error(error: VMError) -> Result<(), VMStatus> {
    let status = error.into_vm_status();
    Err(match status {
        VMStatus::Executed => VMStatus::Executed,
        VMStatus::MoveAbort(location, code) => {
            let (category, reason) = error_split(code);
            let new_major_status = match (category, reason) {
                (INVALID_ARGUMENT, EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY) => {
                    StatusCode::INVALID_AUTH_KEY
                }
                (INVALID_ARGUMENT, EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD) => {
                    StatusCode::SEQUENCE_NUMBER_TOO_OLD
                }
                (INVALID_ARGUMENT, EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW) => {
                    StatusCode::SEQUENCE_NUMBER_TOO_NEW
                }
                (INVALID_ARGUMENT, EPROLOGUE_CANT_PAY_GAS_DEPOSIT) => {
                    StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE
                }
                (INVALID_ARGUMENT, EPROLOGUE_TRANSACTION_EXPIRED) => {
                    StatusCode::TRANSACTION_EXPIRED
                }
                (INVALID_ARGUMENT, EPROLOGUE_BAD_CHAIN_ID) => StatusCode::BAD_CHAIN_ID,
                (INVALID_ARGUMENT, EPROLOGUE_MODULE_NOT_ALLOWED) => {
                    StatusCode::INVALID_MODULE_PUBLISHER
                }
                (INVALID_ARGUMENT, EPROLOGUE_SCRIPT_NOT_ALLOWED) => StatusCode::UNKNOWN_SCRIPT,
                (INVALID_ARGUMENT, EINVALID_TIMESTAMP) => StatusCode::INVALID_TIMESTAMP,
                (INVALID_ARGUMENT, ECOIN_DEPOSIT_IS_ZERO) => StatusCode::COIN_DEPOSIT_IS_ZERO,
                (INVALID_ARGUMENT, EBLOCK_NUMBER_MISMATCH) => StatusCode::BLOCK_NUMBER_MISMATCH,
                (INVALID_ARGUMENT, EBAD_TRANSACTION_FEE_TOKEN) => {
                    StatusCode::BAD_TRANSACTION_FEE_CURRENCY
                }
                // todo: enable these error code if global frozen is enabled
                //(INVALID_ARGUMENT, EPROLOGUE_SENDING_ACCOUNT_FROZEN) => {
                //    StatusCode::SENDING_ACCOUNT_FROZEN
                //}
                //(INVALID_ARGUMENT, EPROLOGUE_SENDING_TXN_GLOBAL_FROZEN) => {
                //    StatusCode::SEND_TXN_GLOBAL_FROZEN
                //}
                (OUT_OF_RANGE, EPROLOGUE_ACCOUNT_DOES_NOT_EXIST) => {
                    StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST
                }
                (OUT_OF_RANGE, EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG) => {
                    StatusCode::SEQUENCE_NUMBER_TOO_BIG
                }
                // todo: unused, double check
                //(OUT_OF_RANGE, ENOT_GENESIS_ACCOUNT) => StatusCode::NO_ACCOUNT_ROLE,
                (INVALID_STATE, ENOT_GENESIS) => StatusCode::NOT_GENESIS,
                (INVALID_STATE, ECONFIG_VALUE_DOES_NOT_EXIST) => {
                    StatusCode::CONFIG_VALUE_DOES_NOT_EXIST
                }
                (INVALID_STATE, EPROLOGUE_SIGNER_ALREADY_DELEGATED) => {
                    StatusCode::SIGNER_ALREADY_DELEGATED
                }
                (INVALID_STATE, EDESTROY_TOKEN_NON_ZERO) => StatusCode::DESTROY_TOKEN_NON_ZERO,

                (category, reason) => {
                    warn!(
                        "prologue runtime unknown: category({}), reason:({}), location:({})",
                        category, reason, location
                    );
                    StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION
                }
            };
            VMStatus::error(new_major_status, None)
        }
        status @ VMStatus::ExecutionFailure { .. } | status @ VMStatus::Error { .. } => {
            error!("[starcoin_vm] Unexpected prologue error: {:?}", status);
            VMStatus::error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION, None)
        }
    })
}

pub fn convert_normal_success_epilogue_error(error: VMError) -> Result<(), VMStatus> {
    let status = error.into_vm_status();
    Err(match status {
        VMStatus::MoveAbort(location, code) => {
            let (category, reason) = error_split(code);
            match (category, reason) {
                (ALREADY_EXISTS, EINSUFFICIENT_BALANCE) => {
                    let _ = location != account_module_abort();
                    VMStatus::MoveAbort(location, code)
                }
                (category, reason) => {
                    error!(
                        "[starcoin_vm] Unexpected success epilogue Move abort: {:?}::{:?} (Category: {:?} Reason: {:?})",
                        location, code, category, reason,
                    );
                    VMStatus::error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION, None)
                }
            }
        }

        status @ VMStatus::Executed => status,

        status => {
            error!(
                "[starcoin_vm] Unexpected success epilogue error: {:?}",
                status
            );
            VMStatus::error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION, None)
        }
    })
}

fn account_module_abort() -> AbortLocation {
    AbortLocation::Module(G_ACCOUNT_MODULE.clone())
}
