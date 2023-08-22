// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters::*, data_cache::StateViewCache};
use anyhow::Result;
use starcoin_state_view::StateView;
use aptos_types::{
    transaction::{SignatureCheckedTransaction, SignedTransaction, VMValidatorResult},
    vm_status::{StatusCode, VMStatus},
};
use move_core_types::resolver::MoveResolver;

use crate::{
    data_cache::AsMoveResolver,
    logging::AdapterLogSchema,
    move_vm_ext::{SessionExt, SessionId},
};
use starcoin_logger::prelude::*;
use starcoin_vm_types::{
    access_path::AccessPath,
    block_metadata::BlockMetadata,
    state_store::state_key::StateKey,
    transaction::{
        Transaction, TransactionArgument, TransactionOutput, TransactionPayload, TransactionStatus,
        WriteSetPayload,SignedUserTransaction,
    },
    write_set::WriteSet,
};
use rayon::prelude::*;
use std::collections::HashSet;
use move_core_types::vm_status::{StatusCode, VMStatus};
use starcoin_vm_types::state_view::StateView;

/// This trait describes the VM adapter's interface.
/// TODO: bring more of the execution logic in aptos_vm into this file.
pub trait VMAdapter {
    /// Creates a new Session backed by the given storage.
    /// TODO: this doesn't belong in this trait. We should be able to remove
    /// this after redesigning cache ownership model.
    fn new_session<'r, R: MoveResolver>(
        &self,
        remote: &'r R,
        session_id: SessionId,
    ) -> SessionExt<'r, '_, R>;

    /// Checks the signature of the given signed transaction and returns
    /// `Ok(SignatureCheckedTransaction)` if the signature is valid.
    fn check_signature(txn: SignedTransaction) -> Result<SignatureCheckedTransaction>;

    /// Check if the transaction format is supported.
    fn check_transaction_format(&self, txn: &SignedTransaction) -> Result<(), VMStatus>;

    /// Get the gas price for the given transaction.
    /// TODO: remove this after making mempool interface more generic so
    /// it ranks transactions using a provided ranker that implements PartialOrd
    /// instead of using governance roles and gas prices.
    fn get_gas_price<S: MoveResolver>(
        &self,
        txn: &SignedTransaction,
        remote_cache: &S,
    ) -> Result<u64, VMStatus>;

    /// Runs the prologue for the given transaction.
    fn run_prologue<S: MoveResolver>(
        &self,
        session: &mut SessionExt<S>,
        transaction: &SignatureCheckedTransaction,
        log_context: &AdapterLogSchema,
    ) -> Result<(), VMStatus>;

    /// TODO: maybe remove this after more refactoring of execution logic.
    fn should_restart_execution(output: &TransactionOutput) -> bool;

    /// Execute a single transaction.
    fn execute_single_transaction<S: MoveResolver + StateView>(
        &self,
        txn: &PreprocessedTransaction,
        data_cache: &S,
        log_context: &AdapterLogSchema,
    ) -> Result<(VMStatus, TransactionOutput, Option<String>), VMStatus>;
}

/// Validate a signed transaction by performing the following:
/// 1. Check the signature(s) included in the signed transaction
/// 2. Check that the transaction is allowed in the context provided by the `adapter`
/// 3. Run the prologue to perform additional on-chain checks
/// The returned `VMValidatorResult` will have status `None` and if all checks succeeded
/// and `Some(DiscardedVMStatus)` otherwise.
pub fn validate_signed_transaction<A: VMAdapter>(
    adapter: &A,
    transaction: SignedTransaction,
    state_view: &impl StateView,
) -> VMValidatorResult {
    let _timer = TXN_VALIDATION_SECONDS.start_timer();
    let log_context = AdapterLogSchema::new(state_view.id(), 0);

    let txn = match A::check_signature(transaction) {
        Ok(t) => t,
        _ => {
            return VMValidatorResult::error(StatusCode::INVALID_SIGNATURE);
        }
    };

    let remote_cache = StateViewCache::new(state_view);
    let resolver = remote_cache.as_move_resolver();
    let mut session = adapter.new_session(&resolver, SessionId::txn(&txn));

    let (status, gas_price) = match adapter.get_gas_price(&*txn, &resolver) {
        Ok(price) => (None, price),
        Err(err) => (Some(err.status_code()), 0),
    };

    let validation_result =
        validate_signature_checked_transaction(adapter, &mut session, &txn, true, &log_context);

    let (status, gas_price) = match (status, validation_result) {
        (Some(_), _) => (status, 0),
        (None, Ok(())) => (None, gas_price),
        (None, Err(err)) => (Some(err.status_code()), 0),
    };

    // Increment the counter for transactions verified.
    let counter_label = match status {
        None => "success",
        Some(_) => "failure",
    };
    TRANSACTIONS_VALIDATED
        .with_label_values(&[counter_label])
        .inc();

    VMValidatorResult::new(status, gas_price)
}

pub(crate) fn validate_signature_checked_transaction<S: MoveResolver, A: VMAdapter>(
    adapter: &A,
    session: &mut SessionExt<S>,
    transaction: &SignatureCheckedTransaction,
    allow_too_new: bool,
    log_context: &AdapterLogSchema,
) -> Result<(), VMStatus> {
    adapter.check_transaction_format(transaction)?;

    let prologue_status = adapter.run_prologue(session, transaction, log_context);
    match prologue_status {
        Err(err) if !allow_too_new || err.status_code() != StatusCode::SEQUENCE_NUMBER_TOO_NEW => {
            Err(err)
        }
        _ => Ok(()),
    }
}

fn preload_cache(signature_verified_block: &[PreprocessedTransaction], data_view: &impl StateView) {
    // generate a collection of addresses
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
}

pub(crate) fn execute_block_impl<A: VMAdapter, S: StateView>(
    adapter: &A,
    transactions: Vec<Transaction>,
    data_cache: &mut StateViewCache<S>,
) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
    let mut result = vec![];
    let mut should_restart = false;

    info!(
        AdapterLogSchema::new(data_cache.id(), 0),
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
            .map(preprocess_transaction::<A>)
            .collect();
    }

    rayon::scope(|scope| {
        scope.spawn(|_| {
            preload_cache(&signature_verified_block, data_cache);
        });
    });

    for (idx, txn) in signature_verified_block.into_iter().enumerate() {
        let log_context = AdapterLogSchema::new(data_cache.id(), idx);
        if should_restart {
            let txn_output =
                TransactionOutput::new(WriteSet::default(), vec![], 0, TransactionStatus::Retry);
            result.push((VMStatus::Error(StatusCode::UNKNOWN_STATUS), txn_output));
            debug!(log_context, "Retry after reconfiguration");
            continue;
        };
        let (vm_status, output, sender) = adapter.execute_single_transaction(
            &txn,
            &data_cache.as_move_resolver(),
            &log_context,
        )?;
        if !output.status().is_discarded() {
            data_cache.push_write_set(output.write_set());
        } else {
            match sender {
                Some(s) => trace!(
                    log_context,
                    "Transaction discarded, sender: {}, error: {:?}",
                    s,
                    vm_status,
                ),
                None => trace!(log_context, "Transaction malformed, error: {:?}", vm_status,),
            }
        }

        if A::should_restart_execution(&output) {
            info!(
                AdapterLogSchema::new(data_cache.id(), 0),
                "Reconfiguration occurred: restart required",
            );
            should_restart = true;
        }

        // `result` is initially empty, a single element is pushed per loop iteration and
        // the number of iterations is bound to the max size of `signature_verified_block`
        assume!(result.len() < usize::max_value());
        result.push((vm_status, output))
    }
    Ok(result)
}

/// Transactions after signature checking:
/// BlockPrologues are not signed and are unaffected by signature checking,
/// but a user transaction transformed to a SignatureCheckedTransaction.
#[derive(Debug)]
pub enum PreprocessedTransaction {
    UserTransaction(Box<SignedUserTransaction>),
    BlockMetadata(BlockMetadata),
}

/// Check the signature (if any) of a transaction. If the signature is OK, the result
/// is a PreprocessedTransaction, where a user transaction is translated to a
/// SignatureCheckedTransaction and also categorized into either a UserTransaction
/// or a WriteSet transaction.
pub(crate) fn preprocess_transaction<A: VMAdapter>(txn: Transaction) -> PreprocessedTransaction {
    match txn {
        Transaction::BlockMetadata(b) => PreprocessedTransaction::BlockMetadata(b),
        Transaction::UserTransaction(txn) => {
            let checked_txn = match A::check_signature(txn) {
                Ok(checked_txn) => checked_txn,
                _ => {
                    return PreprocessedTransaction::InvalidSignature;
                }
            };
            match checked_txn.payload() {
                _ => PreprocessedTransaction::UserTransaction(Box::new(checked_txn)),
            }
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
