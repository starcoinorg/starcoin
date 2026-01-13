// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_vm_types::delayed_values::delayed_field_id::DelayedFieldID;
use starcoin_aggregator::{
    delta_math::DeltaHistory,
    types::{code_invariant_error, DelayedFieldValue, DelayedFieldsSpeculativeError, PanicOr},
};
use starcoin_logger::prelude::info;
use starcoin_mvhashmap::types::MVDelayedFieldsError;
use starcoin_mvhashmap::versioned_delayed_fields::TVersionedDelayedFieldView;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DelayedFieldReadKind {
    HistoryBounded,
    Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DelayedFieldRead {
    Value {
        value: DelayedFieldValue,
    },
    HistoryBounded {
        restriction: DeltaHistory,
        max_value: u128,
        inner_aggregator_value: u128,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataReadComparison {
    Contains,
    Insufficient,
    Inconsistent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DelayedReadValidationError {
    Invalid,
    Fatal,
}

impl DelayedFieldRead {
    fn get_kind(&self) -> DelayedFieldReadKind {
        match self {
            DelayedFieldRead::Value { .. } => DelayedFieldReadKind::Value,
            DelayedFieldRead::HistoryBounded { .. } => DelayedFieldReadKind::HistoryBounded,
        }
    }

    pub fn filter_by_kind(&self, min_kind: DelayedFieldReadKind) -> Option<DelayedFieldRead> {
        let self_kind = self.get_kind();
        if self_kind >= min_kind {
            Some(self.clone())
        } else {
            None
        }
    }

    fn contains(&self, other: &DelayedFieldRead) -> DataReadComparison {
        match (&self, other) {
            (DelayedFieldRead::Value { value: v1 }, DelayedFieldRead::Value { value: v2 }) => {
                if v1 == v2 {
                    DataReadComparison::Contains
                } else {
                    DataReadComparison::Inconsistent
                }
            }
            (
                DelayedFieldRead::HistoryBounded {
                    restriction: h1,
                    max_value: m1,
                    inner_aggregator_value: v1,
                },
                DelayedFieldRead::HistoryBounded {
                    restriction: h2,
                    max_value: m2,
                    inner_aggregator_value: v2,
                },
            ) => {
                if v1 == v2 && m1 == m2 && h1.stricter_than(h2) {
                    DataReadComparison::Contains
                } else {
                    DataReadComparison::Inconsistent
                }
            }
            (DelayedFieldRead::HistoryBounded { .. }, DelayedFieldRead::Value { .. }) => {
                DataReadComparison::Insufficient
            }
            (
                DelayedFieldRead::Value { value: v1 },
                DelayedFieldRead::HistoryBounded {
                    restriction,
                    max_value,
                    ..
                },
            ) => {
                if let Ok(v1) = v1.clone().into_aggregator_value() {
                    if restriction
                        .validate_against_base_value(v1, *max_value)
                        .is_ok()
                    {
                        DataReadComparison::Contains
                    } else {
                        DataReadComparison::Inconsistent
                    }
                } else {
                    DataReadComparison::Inconsistent
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct DelayedFieldReads {
    reads: HashMap<DelayedFieldID, DelayedFieldRead>,
    speculative_failure: bool,
    incorrect_use: bool,
}

impl DelayedFieldReads {
    pub fn capture_delayed_field_read(
        &mut self,
        id: DelayedFieldID,
        update: bool,
        read: DelayedFieldRead,
    ) -> Result<(), PanicOr<DelayedFieldsSpeculativeError>> {
        use std::collections::hash_map::Entry::{Occupied, Vacant};

        let result = match self.reads.entry(id) {
            Vacant(e) => {
                e.insert(read);
                Ok(())
            }
            Occupied(mut e) => {
                let existing_read = e.get_mut();
                let read_kind = read.get_kind();
                let existing_kind = existing_read.get_kind();
                if read_kind < existing_kind || (!update && read_kind == existing_kind) {
                    self.incorrect_use = true;
                    Err(code_invariant_error(format!(
                        "Incorrect use capture_delayed_field_read, read {:?}, existing {:?}",
                        read, existing_read
                    ))
                    .into())
                } else {
                    match read.contains(existing_read) {
                        DataReadComparison::Contains => {
                            *existing_read = read;
                            Ok(())
                        }
                        DataReadComparison::Inconsistent => {
                            self.speculative_failure = true;
                            Err(PanicOr::Or(DelayedFieldsSpeculativeError::InconsistentRead))
                        }
                        DataReadComparison::Insufficient => {
                            self.incorrect_use = true;
                            Err(code_invariant_error(format!(
                                "Incorrect use capture_delayed_field_read, {:?} insufficient for {:?}",
                                read, existing_read
                            ))
                            .into())
                        }
                    }
                }
            }
        };

        result
    }

    pub fn capture_delayed_field_read_error<E: std::fmt::Debug>(&mut self, e: &PanicOr<E>) {
        match e {
            PanicOr::CodeInvariantError(_) => self.incorrect_use = true,
            PanicOr::Or(_) => self.speculative_failure = true,
        };
    }

    pub fn is_incorrect_use(&self) -> bool {
        self.incorrect_use
    }

    pub fn has_speculative_failure(&self) -> bool {
        self.speculative_failure
    }

    pub fn get_delayed_field_by_kind(
        &self,
        id: &DelayedFieldID,
        min_kind: DelayedFieldReadKind,
    ) -> Option<DelayedFieldRead> {
        self.reads.get(id).and_then(|r| r.filter_by_kind(min_kind))
    }

    pub fn take(self) -> HashMap<DelayedFieldID, DelayedFieldRead> {
        self.reads
    }

    pub fn validate_delayed_field_reads(
        &self,
        delayed_fields: &dyn TVersionedDelayedFieldView<DelayedFieldID>,
        txn_idx: usize,
    ) -> Result<(), DelayedReadValidationError> {
        if self.incorrect_use {
            info!(
                target: "starcoin_parallel_executor",
                "delayed read incorrect use txn_idx={}",
                txn_idx
            );
            return Err(DelayedReadValidationError::Fatal);
        }
        if self.speculative_failure {
            info!(
                target: "starcoin_parallel_executor",
                "delayed read speculative failure txn_idx={}",
                txn_idx
            );
            return Err(DelayedReadValidationError::Invalid);
        }
        for (id, read_value) in &self.reads {
            let latest_value = match delayed_fields.read_latest_predicted_value(
                id,
                txn_idx,
                starcoin_aggregator::types::ReadPosition::BeforeCurrentTxn,
            ) {
                Ok(v) => v,
                Err(MVDelayedFieldsError::NotFound)
                | Err(MVDelayedFieldsError::Dependency(_))
                | Err(MVDelayedFieldsError::DeltaApplicationFailure) => {
                    info!(
                        target: "starcoin_parallel_executor",
                        "delayed read failed txn_idx={} id={:?}",
                        txn_idx,
                        id
                    );
                    return Err(DelayedReadValidationError::Invalid);
                }
            };
            match read_value {
                DelayedFieldRead::Value { value } => {
                    if value != &latest_value {
                        info!(
                            target: "starcoin_parallel_executor",
                            "delayed read mismatch txn_idx={} id={:?} read={:?} latest={:?}",
                            txn_idx,
                            id,
                            value,
                            latest_value
                        );
                        return Err(DelayedReadValidationError::Invalid);
                    }
                }
                DelayedFieldRead::HistoryBounded {
                    restriction,
                    max_value,
                    ..
                } => {
                    let latest = match latest_value.clone().into_aggregator_value() {
                        Ok(value) => value,
                        Err(err) => {
                            info!(
                                target: "starcoin_parallel_executor",
                                "delayed read invalid latest value txn_idx={} id={:?} latest={:?} err={:?}",
                                txn_idx,
                                id,
                                latest_value,
                                err
                            );
                            return Err(DelayedReadValidationError::Fatal);
                        }
                    };
                    if restriction
                        .validate_against_base_value(latest, *max_value)
                        .is_err()
                    {
                        info!(
                            target: "starcoin_parallel_executor",
                            "delayed read history invalid txn_idx={} id={:?} latest={} max_value={}",
                            txn_idx,
                            id,
                            latest,
                            max_value
                        );
                        return Err(DelayedReadValidationError::Invalid);
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use move_vm_types::delayed_values::delayed_field_id::DelayedFieldID;
    use starcoin_aggregator::bounded_math::SignedU128;
    use starcoin_aggregator::delayed_change::{DelayedApplyEntry, DelayedEntry};
    use starcoin_aggregator::delta_change_set::DeltaWithMax;
    use starcoin_mvhashmap::versioned_delayed_fields::VersionedDelayedFields;

    #[test]
    fn validate_reads_sees_speculative_delta() {
        let id = DelayedFieldID::new_with_width(1, 8);
        let fields = VersionedDelayedFields::empty();
        fields.set_base_value(id, DelayedFieldValue::Aggregator(10));
        let change = DelayedEntry::Apply(DelayedApplyEntry::AggregatorDelta {
            delta: DeltaWithMax::new(SignedU128::Positive(5), 100).into_op_no_additional_history(),
        });
        fields.record_change(id, 0, change).unwrap();
        fields.try_commit(0, std::iter::once(id)).unwrap();

        let mut reads = DelayedFieldReads::default();
        reads
            .capture_delayed_field_read(
                id,
                false,
                DelayedFieldRead::Value {
                    value: DelayedFieldValue::Aggregator(15),
                },
            )
            .unwrap();

        let result = reads.validate_delayed_field_reads(&fields, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_reads_detects_mismatch() {
        let id = DelayedFieldID::new_with_width(1, 8);
        let fields = VersionedDelayedFields::empty();
        fields.set_base_value(id, DelayedFieldValue::Aggregator(10));

        let mut reads = DelayedFieldReads::default();
        reads
            .capture_delayed_field_read(
                id,
                false,
                DelayedFieldRead::Value {
                    value: DelayedFieldValue::Aggregator(11),
                },
            )
            .unwrap();

        let result = reads.validate_delayed_field_reads(&fields, 0);
        assert!(matches!(
            result,
            Err(DelayedReadValidationError::Invalid)
        ));
    }

    #[test]
    fn validate_reads_incorrect_use_is_fatal() {
        let id = DelayedFieldID::new_with_width(2, 8);
        let fields = VersionedDelayedFields::empty();
        fields.set_base_value(id, DelayedFieldValue::Aggregator(10));

        let mut reads = DelayedFieldReads::default();
        let _ = reads.capture_delayed_field_read(
            id,
            false,
            DelayedFieldRead::Value {
                value: DelayedFieldValue::Aggregator(10),
            },
        );
        let _ = reads.capture_delayed_field_read(
            id,
            false,
            DelayedFieldRead::Value {
                value: DelayedFieldValue::Aggregator(10),
            },
        );
        assert!(reads.is_incorrect_use());

        let result = reads.validate_delayed_field_reads(&fields, 0);
        assert!(matches!(
            result,
            Err(DelayedReadValidationError::Fatal)
        ));
    }
}
