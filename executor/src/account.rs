// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Test infrastructure for modeling Diem accounts.

use crate::{create_signed_txn_with_association_account, DEFAULT_MAX_GAS_AMOUNT};
use starcoin_config::ChainNetwork;
use starcoin_types::transaction::{ScriptFunction, SignedUserTransaction, TransactionPayload};
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::account_config::stc_type_tag;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::ModuleId;

// TTL is 86400s. Initial time was set to 0.
pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;
pub use starcoin_types::account::*;

/// Returns a transaction to transfer coin from one account to another (possibly new) one, with the
/// given arguments.
#[allow(clippy::vec_init_then_push)]
pub fn peer_to_peer_txn(
    sender: &Account,
    receiver: &Account,
    seq_num: u64,
    transfer_amount: u128,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> SignedUserTransaction {
    let mut args: Vec<Vec<u8>> = Vec::new();
    args.push(bcs_ext::to_bytes(receiver.address()).unwrap());
    args.push(bcs_ext::to_bytes(&transfer_amount).unwrap());

    // get a SignedTransaction
    sender.create_signed_txn_with_args(
        TransactionPayload::ScriptFunction(ScriptFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("TransferScripts").unwrap(),
            ),
            Identifier::new("peer_to_peer_v2").unwrap(),
            vec![stc_type_tag()],
            args,
        )),
        seq_num,
        DEFAULT_MAX_GAS_AMOUNT, // this is a default for gas
        1,                      // this is a default for gas
        expiration_timestamp_secs,
        chain_id,
    )
}

/// Returns a transaction to create a new account with the given arguments.
#[allow(clippy::vec_init_then_push)]
pub fn create_account_txn_sent_as_association(
    new_account: &Account,
    seq_num: u64,
    initial_amount: u128,
    expiration_timstamp_secs: u64,
    net: &ChainNetwork,
) -> SignedUserTransaction {
    let mut args: Vec<Vec<u8>> = Vec::new();
    args.push(bcs_ext::to_bytes(new_account.address()).unwrap());
    args.push(bcs_ext::to_bytes(&new_account.auth_key().to_vec()).unwrap());
    args.push(bcs_ext::to_bytes(&initial_amount).unwrap());

    create_signed_txn_with_association_account(
        TransactionPayload::ScriptFunction(ScriptFunction::new(
            ModuleId::new(core_code_address(), Identifier::new("Account").unwrap()),
            Identifier::new("create_account_with_initial_amount").unwrap(),
            vec![stc_type_tag()],
            args,
        )),
        seq_num,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        expiration_timstamp_secs,
        net,
    )
}
