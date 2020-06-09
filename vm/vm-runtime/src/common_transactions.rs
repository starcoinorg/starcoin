// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for encoding transactions for common situations.

use crate::genesis::GENESIS_KEYPAIR;
use starcoin_types::transaction::{
    RawUserTransaction, Script, SignedUserTransaction, TransactionArgument, TransactionPayload,
};
use starcoin_vm_types::account_config;
use starcoin_vm_types::language_storage::TypeTag;
use std::time::Duration;

//TODO move to transaction_builder crate.
pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;
pub const TXN_RESERVED: u64 = 2_000_000;

pub fn create_signed_txn_with_association_account(
    program: Vec<u8>,
    ty_args: Vec<TypeTag>,
    args: Vec<TransactionArgument>,
    sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
) -> SignedUserTransaction {
    RawUserTransaction::new(
        account_config::association_address(),
        sequence_number,
        TransactionPayload::Script(Script::new(program, ty_args, args)),
        max_gas_amount,
        gas_unit_price,
        // TTL is 86400s. Initial time was set to 0.
        Duration::from_secs(DEFAULT_EXPIRATION_TIME),
    )
    .sign(&GENESIS_KEYPAIR.0, GENESIS_KEYPAIR.1.clone())
    .unwrap()
    .into_inner()
}
