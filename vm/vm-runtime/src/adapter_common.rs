// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_cache::StateViewCache,
    move_vm_ext::{MoveResolverExt, SessionId},
};
use anyhow::Result;
use move_core_types::vm_status::{StatusCode, VMStatus};
use move_vm_runtime::move_vm_adapter::SessionAdapter;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::{
    block_metadata::BlockMetadata,
    transaction::{
        SignatureCheckedTransaction, SignedUserTransaction, Transaction, TransactionOutput,
        TransactionStatus,
    },
    write_set::WriteSet,
};

/// This trait describes the VM adapter's interface.
/// TODO: bring more of the execution logic in aptos_vm into this file.
pub trait VMAdapter {
    /// Creates a new Session backed by the given storage.
    /// TODO: this doesn't belong in this trait. We should be able to remove
    /// this after redesigning cache ownership model.
    // XXX FIXME YSG, this place we use SessionAdapter, we don't have move_vm_ext::SessionExt
    /// XXX FIXME YSG, we don't know
    fn new_session<'r, R: MoveResolverExt>(
        &self,
        remote: &'r R,
        session_id: SessionId,
    ) -> SessionAdapter<'r, '_, R>;

    /// Checks the signature of the given signed transaction and returns
    /// `Ok(SignatureCheckedTransaction)` if the signature is valid.
    fn check_signature(txn: SignedUserTransaction) -> Result<SignatureCheckedTransaction>;

    /// TODO: maybe remove this after more refactoring of execution logic.
    fn should_restart_execution(output: &TransactionOutput) -> bool;

    /// Execute a single transaction.
    fn execute_single_transaction<S: MoveResolverExt + StateView>(
        &self,
        txn: &PreprocessedTransaction,
        data_cache: &S,
    ) -> Result<(VMStatus, TransactionOutput, Option<String>), VMStatus>;
}

#[allow(dead_code)]
fn preload_cache(
    _signature_verified_block: &[PreprocessedTransaction],
    _data_view: &impl StateView,
) {
    // XXX FIXME YSG
    // generate a collection of addresses
    /*
    let mut addresses_to_preload = HashSet::new();
    for txn in signature_verified_block {
        if let PreprocessedTransaction::UserTransaction(txn) = txn {
            if let TransactionPayload::Script(script) = txn.payload() {
                addresses_to_preload.insert(txn.sender());

                for arg in script.args() {
                    if let TransactionArgument::Address(address) = arg {
                        addresses_to_preload.insert(*address);
                    }
                }
            }
        }
    }

    // This will launch a number of threads to preload the account blobs in parallel. We may
    // want to fine tune the number of threads launched here in the future.
    addresses_to_preload
        .into_par_iter()
        .map(|addr| {
            data_view
                .get_state_value(&StateKey::AccessPath(AccessPath::new(addr, Vec::new())))
                .ok()?
        })
        .collect::<Vec<Option<Vec<u8>>>>();
     */
}

#[allow(dead_code)]
pub(crate) fn execute_block_impl<A: VMAdapter, S: StateView>(
    _adapter: &A,
    _transactions: Vec<Transaction>,
    _data_cache: &mut StateViewCache<S>,
) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
    let result = vec![];
    // XXX FIXME YSG, need open it
    // check if preload_cache can use in
    /*
    let mut should_restart = false;

    info!(
        "Executing block, transaction count: {}",
        transactions.len()
    );

    let signature_verified_block: Vec<PreprocessedTransaction>;
    {
        // Verify the signatures of all the transactions in parallel.
        // This is time consuming so don't wait and do the checking
        // sequentially while executing the transactions.
        signature_verified_block = transactions
            .into_par_iter()
            .map(preprocess_transaction)
            .collect();
    }

    rayon::scope(|scope| {
        scope.spawn(|_| {
            preload_cache(&signature_verified_block, data_cache);
        });
    });

    for (idx, txn) in signature_verified_block.into_iter().enumerate() {
        if should_restart {
            let txn_output =
                TransactionOutput::new(WriteSet::default(), vec![], 0, TransactionStatus::Retry);
            result.push((VMStatus::Error(StatusCode::UNKNOWN_STATUS), txn_output));
            debug!("Retry after reconfiguration");
            continue;
        };
        let (vm_status, output, sender) = adapter.execute_single_transaction(
            &txn,
            &data_cache.as_move_resolver(),
        )?;
        if !output.status().is_discarded() {
            data_cache.push_write_set(output.write_set());
        } else {
            match sender {
                Some(s) => trace!(
                    "Transaction discarded, sender: {}, error: {:?}",
                    s,
                    vm_status,
                ),
                None => trace!("Transaction malformed, error: {:?}", vm_status,),
            }
        }

        if A::should_restart_execution(&output) {
            info!(
                "Reconfiguration occurred: restart required",
            );
            should_restart = true;
        }

        // `result` is initially empty, a single element is pushed per loop iteration and
        // the number of iterations is bound to the max size of `signature_verified_block`
        assume!(result.len() < usize::max_value());
        result.push((vm_status, output))
    } */
    Ok(result)
}

#[derive(Debug)]
pub enum PreprocessedTransaction {
    UserTransaction(Box<SignedUserTransaction>),
    BlockMetadata(BlockMetadata),
}

#[inline]
pub(crate) fn preprocess_transaction(txn: Transaction) -> PreprocessedTransaction {
    match txn {
        Transaction::BlockMetadata(b) => PreprocessedTransaction::BlockMetadata(b),
        Transaction::UserTransaction(txn) => {
            PreprocessedTransaction::UserTransaction(Box::new(txn))
        }
    }
}

pub(crate) fn discard_error_vm_status(err: VMStatus) -> (VMStatus, TransactionOutput) {
    let vm_status = err.clone();
    let error_code = match err.keep_or_discard() {
        Ok(_) => {
            debug_assert!(false, "discarding non-discardable error: {:?}", vm_status);
            vm_status.status_code()
        }
        Err(code) => code,
    };
    (vm_status, discard_error_output(error_code))
}

pub(crate) fn discard_error_output(err: StatusCode) -> TransactionOutput {
    // Since this transaction will be discarded, no writeset will be included.
    TransactionOutput::new(
        WriteSet::default(),
        vec![],
        0,
        TransactionStatus::Discard(err),
    )
}
