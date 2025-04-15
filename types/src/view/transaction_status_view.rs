// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::str_view::StrView;
use move_core_types::vm_status::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::{
    transaction::TransactionStatus,
    vm_status::{AbortLocation, DiscardedVMStatus, KeptVMStatus},
};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
pub enum TransactionStatusView {
    Executed,
    OutOfGas,

    MoveAbort {
        //Todo: remote define it
        #[schemars(with = "String")]
        location: AbortLocation,
        abort_code: StrView<u64>,
    },
    ExecutionFailure {
        #[schemars(with = "String")]
        location: AbortLocation,
        function: u16,
        code_offset: u16,
    },
    MiscellaneousError,
    Discard {
        status_code: StrView<u64>,
        status_code_name: String,
    },
    Retry,
}

impl From<TransactionStatus> for TransactionStatusView {
    fn from(s: TransactionStatus) -> Self {
        match s {
            TransactionStatus::Discard(d) => d.into(),
            TransactionStatus::Keep(k) => k.into(),
            TransactionStatus::Retry => Self::Retry,
        }
    }
}

impl From<KeptVMStatus> for TransactionStatusView {
    fn from(origin: KeptVMStatus) -> Self {
        match origin {
            KeptVMStatus::Executed => Self::Executed,
            KeptVMStatus::OutOfGas => Self::OutOfGas,
            KeptVMStatus::MoveAbort(l, c) => Self::MoveAbort {
                location: l,
                abort_code: c.into(),
            },
            KeptVMStatus::ExecutionFailure {
                location,
                function,
                code_offset,
                ..
            } => Self::ExecutionFailure {
                location,
                function,
                code_offset,
            },
            KeptVMStatus::MiscellaneousError => Self::MiscellaneousError,
        }
    }
}

impl From<DiscardedVMStatus> for TransactionStatusView {
    fn from(s: DiscardedVMStatus) -> Self {
        Self::Discard {
            status_code: StrView(s.into()),
            status_code_name: format!("{:?}", s),
        }
    }
}

impl From<TransactionStatusView> for TransactionStatus {
    fn from(value: TransactionStatusView) -> Self {
        match value {
            TransactionStatusView::Executed => Self::Keep(KeptVMStatus::Executed),
            TransactionStatusView::OutOfGas => Self::Keep(KeptVMStatus::OutOfGas),
            TransactionStatusView::MoveAbort {
                location,
                abort_code,
            } => Self::Keep(KeptVMStatus::MoveAbort(location, abort_code.0)),
            TransactionStatusView::MiscellaneousError => {
                Self::Keep(KeptVMStatus::MiscellaneousError)
            }
            TransactionStatusView::ExecutionFailure {
                location,
                function,
                code_offset,
            } => Self::Keep(KeptVMStatus::ExecutionFailure {
                location,
                function,
                code_offset,
                message: None,
            }),
            TransactionStatusView::Discard {
                status_code,
                status_code_name: _,
            } => Self::Discard(
                status_code
                    .0
                    .try_into()
                    .ok()
                    .unwrap_or(StatusCode::UNKNOWN_STATUS),
            ),
            TransactionStatusView::Retry => Self::Retry,
        }
    }
}
