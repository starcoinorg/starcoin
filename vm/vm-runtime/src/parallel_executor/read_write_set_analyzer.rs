// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    adapter_common::PreprocessedTransaction, read_write_set_analysis::ReadWriteSetAnalysis,
};
use anyhow::Result;
use starcoin_parallel_executor::task::{Accesses, ReadWriteSetInferencer};
use starcoin_vm_types::{access_path::AccessPath, state_store::state_key::StateKey};
use move_core_types::resolver::MoveResolver;
use read_write_set_dynamic::NormalizedReadWriteSetAnalysis;

pub(crate) struct ReadWriteSetAnalysisWrapper<'a, S: MoveResolver> {
    analyzer: ReadWriteSetAnalysis<'a, S>,
}
#[allow(dead_code)]
impl<'a, S: MoveResolver> ReadWriteSetAnalysisWrapper<'a, S> {
    pub fn new(analysis_result: &'a NormalizedReadWriteSetAnalysis, view: &'a S) -> Self {
        Self {
            analyzer: ReadWriteSetAnalysis::new(analysis_result, view),
        }
    }
}

impl<'a, S: MoveResolver + std::marker::Sync> ReadWriteSetInferencer
for ReadWriteSetAnalysisWrapper<'a, S>
{
    type T = PreprocessedTransaction;
    fn infer_reads_writes(&self, txn: &Self::T) -> Result<Accesses<StateKey>> {
        let (keys_read, keys_written) = self.analyzer.get_keys_transaction(txn, false)?;
        // TODO: Add support for table items as state key.
        Ok(Accesses {
            keys_read: keys_read
                .into_iter()
                .map(|x| StateKey::AccessPath(AccessPath::resource_access_path(x)))
                .collect(),
            keys_written: keys_written
                .into_iter()
                .map(|x| StateKey::AccessPath(AccessPath::resource_access_path(x)))
                .collect(),
        })
    }
}
