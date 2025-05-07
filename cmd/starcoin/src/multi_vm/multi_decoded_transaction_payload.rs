// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_abi_decoder::DecodedTransactionPayload;
use starcoin_vm2_abi_decoder::DecodedTransactionPayload as DecodedTransactionPayloadV2;

pub enum MultiDecodedTransactionPayload {
    VM1(DecodedTransactionPayload),
    VM2(DecodedTransactionPayloadV2),
}