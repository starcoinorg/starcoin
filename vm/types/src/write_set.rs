// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! For each transaction the VM executes, the VM will output a `WriteSet` that contains each access
//! path it updates. For each access path, the VM can either give its new value or delete it.

use crate::state_store::state_key::StateKey;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum WriteOp {
    Creation(#[serde(with = "serde_bytes")] Vec<u8>),
    Modification(#[serde(with = "serde_bytes")] Vec<u8>),
    Deletion,
}

impl WriteOp {
    #[inline]
    pub fn is_deletion(&self) -> bool {
        match self {
            WriteOp::Deletion => true,
            WriteOp::Creation(_) | WriteOp::Modification(_) => false,
        }
    }

    #[inline]
    pub fn is_creation(&self) -> bool {
        match self {
            WriteOp::Deletion | WriteOp::Modification(_) => false,
            WriteOp::Creation(_) => true,
        }
    }

    #[inline]
    pub fn is_modification(&self) -> bool {
        match self {
            WriteOp::Deletion | WriteOp::Creation(_) => false,
            WriteOp::Modification(_) => true,
        }
    }
}

impl std::fmt::Debug for WriteOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteOp::Creation(value) => write!(
                f,
                "Creation({})",
                value
                    .iter()
                    .map(|byte| format!("{:02x}", byte))
                    .collect::<String>()
            ),
            WriteOp::Modification(value) => write!(
                f,
                "Modification({})",
                value
                    .iter()
                    .map(|byte| format!("{:02x}", byte))
                    .collect::<String>()
            ),
            WriteOp::Deletion => write!(f, "Deletion"),
        }
    }
}

/// `WriteSet` contains all StateKey that one transaction modifies. Each of them is a `WriteOp`
/// where `Value(val)` means that serialized representation should be updated to `val`, and
/// `Deletion` means that we are going to delete this access path.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct WriteSet(WriteSetMut);

impl WriteSet {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> ::std::slice::Iter<'_, (StateKey, WriteOp)> {
        self.into_iter()
    }

    #[inline]
    pub fn into_mut(self) -> WriteSetMut {
        self.0
    }
}

/// A mutable version of `WriteSet`.
///
/// This is separate because it goes through validation before becoming an immutable `WriteSet`.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct WriteSetMut {
    write_set: Vec<(StateKey, WriteOp)>,
}

impl WriteSetMut {
    pub fn new(write_set: Vec<(StateKey, WriteOp)>) -> Self {
        Self { write_set }
    }

    pub fn push(&mut self, item: (StateKey, WriteOp)) {
        self.write_set.push(item);
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.write_set.is_empty()
    }

    pub fn freeze(self) -> Result<WriteSet> {
        // TODO: add structural validation
        Ok(WriteSet(self))
    }
}

impl<'a> IntoIterator for &'a WriteSet {
    type Item = &'a (StateKey, WriteOp);
    type IntoIter = ::std::slice::Iter<'a, (StateKey, WriteOp)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.write_set.iter()
    }
}

impl ::std::iter::IntoIterator for WriteSet {
    type Item = (StateKey, WriteOp);
    type IntoIter = ::std::vec::IntoIter<(StateKey, WriteOp)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.write_set.into_iter()
    }
}
