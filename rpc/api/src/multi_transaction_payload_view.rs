// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2


use crate::types::TransactionPayloadView;
use starcoin_vm2_types::view::transaction_payload_view::TransactionPayloadView as TransactionPayloadViewV2;

pub enum MultiTransactionPayloadView {
    VM1(TransactionPayloadView),
    VM2(TransactionPayloadViewV2),
}


