// Copyright (c) The Starcoin Core Contributors

// this is a ref in aptos-move/aptos-vm-types/src/change_set.rs
use crate::abstract_write_op::AbstractResourceWriteOp;
use crate::check_change_set::CheckChangeSet;
use move_core_types::vm_status::VMStatus;
use starcoin_vm_types::{
    contract_event::ContractEvent,
    state_store::state_key::StateKey,
    transaction::ChangeSet as StorageChangeSet,
    value::MoveTypeLayout,
    write_set::{WriteOp, WriteSetMut},
};
use std::collections::BTreeMap;

/// A change set produced by the VM.
///
/// **WARNING**: Just like VMOutput, this type should only be used inside the
/// VM. For storage backends, use `ChangeSet`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VMChangeSet {
    resource_write_set: BTreeMap<StateKey, AbstractResourceWriteOp>,
    module_write_set: BTreeMap<StateKey, WriteOp>,
    events: Vec<(ContractEvent, Option<MoveTypeLayout>)>,
}

impl VMChangeSet {
    pub fn empty() -> Self {
        Self {
            resource_write_set: BTreeMap::new(),
            module_write_set: BTreeMap::new(),
            events: vec![],
        }
    }

    pub fn new(
        resource_write_set: BTreeMap<StateKey, AbstractResourceWriteOp>,
        module_write_set: BTreeMap<StateKey, WriteOp>,
        events: Vec<(ContractEvent, Option<MoveTypeLayout>)>,
        checker: &dyn CheckChangeSet,
    ) -> Result<Self, VMStatus> {
        let change_set = Self {
            resource_write_set,
            module_write_set,
            events,
        };
        // Returns an error if structure of the change set is not valid,
        // e.g. the size in bytes is too large.
        checker.check_change_set(&change_set)?;
        Ok(change_set)
    }

    /// Converts VM-native change set into its storage representation with fully
    /// serialized changes.
    pub fn try_into_storage_change_set(self) -> Result<StorageChangeSet, VMStatus> {
        let Self {
            resource_write_set,
            module_write_set,
            events,
        } = self;
        let mut write_set_mut = WriteSetMut::default();
        resource_write_set.into_iter().for_each(
            |(access_path, AbstractResourceWriteOp::Write(write_op))| {
                write_set_mut.push((access_path, write_op));
            },
        );
        module_write_set
            .into_iter()
            .for_each(|(access_path, write_op)| {
                write_set_mut.push((access_path, write_op));
            });
        let write_set = write_set_mut
            .freeze()
            .expect("Freezing a WriteSet does not fail");
        let events = events.into_iter().map(|(e, _)| e).collect();
        Ok(StorageChangeSet::new(write_set, events))
    }
}
