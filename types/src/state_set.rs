// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

/// StateSet is represent a single state-tree or sub state-tree dump result.
#[derive(Debug, Default, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct StateSet(Vec<(HashValue, Vec<u8>)>);

impl StateSet {
    pub fn new(states: Vec<(HashValue, Vec<u8>)>) -> Self {
        Self(states)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> ::std::slice::Iter<(HashValue, Vec<u8>)> {
        self.into_iter()
    }

    fn push(&mut self, hash: HashValue, blob: Vec<u8>) {
        //TODO check repeat value ?
        self.0.push((hash, blob))
    }
}

impl ::std::iter::FromIterator<(HashValue, Vec<u8>)> for StateSet {
    fn from_iter<I: IntoIterator<Item = (HashValue, Vec<u8>)>>(iter: I) -> Self {
        let mut s = StateSet::default();
        for write in iter {
            s.push(write.0, write.1);
        }
        s
    }
}

impl<'a> IntoIterator for &'a StateSet {
    type Item = &'a (HashValue, Vec<u8>);
    type IntoIter = ::std::slice::Iter<'a, (HashValue, Vec<u8>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct AccountStateSet {
    code_set: Option<StateSet>,
    resource_set: StateSet,
}

impl AccountStateSet {
    pub fn new(code_set: Option<StateSet>, resource_set: StateSet) -> Self {
        Self {
            code_set,
            resource_set,
        }
    }

    pub fn resource_set(&self) -> &StateSet {
        &self.resource_set
    }

    pub fn code_set(&self) -> Option<&StateSet> {
        self.code_set.as_ref()
    }
}

/// GlobalStateSet is represent global state db dump result.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct GlobalStateSet {
    /// AccountAddress hash to StateSet
    state_sets: Vec<(HashValue, AccountStateSet)>,
    //TODO should include events?
    //events: Vec<ContractEvent>,
}

impl GlobalStateSet {
    pub fn new(state_sets: Vec<(HashValue, AccountStateSet)>) -> Self {
        Self { state_sets }
    }

    pub fn into_inner(self) -> (Vec<(HashValue, AccountStateSet)>) {
        (self.state_sets)
    }

    pub fn state_sets(&self) -> &[(HashValue, AccountStateSet)] {
        &self.state_sets
    }
}

impl<'a> IntoIterator for &'a GlobalStateSet {
    type Item = &'a (HashValue, AccountStateSet);
    type IntoIter = ::std::slice::Iter<'a, (HashValue, AccountStateSet)>;

    fn into_iter(self) -> Self::IntoIter {
        self.state_sets.iter()
    }
}
