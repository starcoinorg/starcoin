// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::DataType;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::account_address::AccountAddress;

/// StateSet is represent a single state-tree or sub state-tree dump result.
#[derive(Debug, Default, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct StateSet(Vec<(Vec<u8>, Vec<u8>)>);

impl StateSet {
    pub fn new(states: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        Self(states)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> ::std::slice::Iter<(Vec<u8>, Vec<u8>)> {
        self.into_iter()
    }

    fn push(&mut self, key: Vec<u8>, blob: Vec<u8>) {
        self.0.push((key, blob))
    }
}

impl ::std::iter::FromIterator<(Vec<u8>, Vec<u8>)> for StateSet {
    fn from_iter<I: IntoIterator<Item = (Vec<u8>, Vec<u8>)>>(iter: I) -> Self {
        let mut s = StateSet::default();
        for write in iter {
            s.push(write.0, write.1);
        }
        s
    }
}

impl<'a> IntoIterator for &'a StateSet {
    type Item = &'a (Vec<u8>, Vec<u8>);
    type IntoIter = ::std::slice::Iter<'a, (Vec<u8>, Vec<u8>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[allow(clippy::from_over_into)]
impl Into<Vec<(Vec<u8>, Vec<u8>)>> for StateSet {
    fn into(self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.0
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct AccountStateSet(Vec<Option<StateSet>>);

impl AccountStateSet {
    pub fn new(state_sets: Vec<Option<StateSet>>) -> Self {
        Self(state_sets)
    }

    pub fn resource_set(&self) -> Option<&StateSet> {
        self.data_set(DataType::RESOURCE)
    }

    pub fn code_set(&self) -> Option<&StateSet> {
        self.data_set(DataType::CODE)
    }

    #[inline]
    pub fn data_set(&self, data_type: DataType) -> Option<&StateSet> {
        self.0[data_type.storage_index()].as_ref()
    }
}

impl<'a> IntoIterator for &'a AccountStateSet {
    type Item = &'a Option<StateSet>;
    type IntoIter = ::std::slice::Iter<'a, Option<StateSet>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// ChainStateSet is represent ChainState dump result.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChainStateSet {
    state_sets: Vec<(AccountAddress, AccountStateSet)>,
    //TODO should include events?
    //events: Vec<ContractEvent>,
}

impl ChainStateSet {
    pub fn new(state_sets: Vec<(AccountAddress, AccountStateSet)>) -> Self {
        Self { state_sets }
    }

    pub fn into_inner(self) -> Vec<(AccountAddress, AccountStateSet)> {
        self.state_sets
    }

    pub fn state_sets(&self) -> &[(AccountAddress, AccountStateSet)] {
        &self.state_sets
    }

    pub fn len(&self) -> usize {
        self.state_sets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.state_sets.is_empty()
    }
}

impl<'a> IntoIterator for &'a ChainStateSet {
    type Item = &'a (AccountAddress, AccountStateSet);
    type IntoIter = ::std::slice::Iter<'a, (AccountAddress, AccountStateSet)>;

    fn into_iter(self) -> Self::IntoIter {
        self.state_sets.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_set() {
        let account_state_set = AccountStateSet::new(vec![None, None]);
        assert_eq!(2, account_state_set.into_iter().count());
    }
}
