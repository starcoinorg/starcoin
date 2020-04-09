// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use merkle_tree::{blob::Blob, proof::SparseMerkleProof};
use serde::{Deserialize, Serialize};
use starcoin_crypto::{hash::CryptoHash, HashValue};
use starcoin_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{AccountResource, BalanceResource},
    account_state::AccountState,
    state_set::ChainStateSet,
};
use std::convert::TryFrom;

#[derive(Debug, Default, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct StateProof {
    account_state: Option<Blob>,
    account_proof: SparseMerkleProof,
    account_state_proof: SparseMerkleProof,
}

impl StateProof {
    pub fn new(
        account_state: Option<Vec<u8>>,
        account_proof: SparseMerkleProof,
        account_state_proof: SparseMerkleProof,
    ) -> Self {
        Self {
            account_state: account_state.map(|data| Blob::from(data)),
            account_proof,
            account_state_proof,
        }
    }
    /// verify the resource blob with `access_path`,
    /// given expected_root_hash, and expected account state blob.
    pub fn verify(
        &self,
        expected_root_hash: HashValue,
        access_path: AccessPath,
        access_resource_blob: Option<&[u8]>,
    ) -> Result<()> {
        let (account_address, data_type, ap_hash) = access_path.into();
        match self.account_state.as_ref() {
            None => {
                ensure!(
                    access_resource_blob.is_none(),
                    "accessed resource should not exists"
                );
            }
            Some(s) => {
                let account_state = AccountState::try_from(s.as_ref())?;
                match account_state.storage_roots()[data_type.storage_index()] {
                    None => {
                        ensure!(
                            access_resource_blob.is_none(),
                            "accessed resource should not exists"
                        );
                    }
                    Some(expected_hash) => {
                        let blob = access_resource_blob.map(|data| Blob::from(data.to_vec()));
                        self.account_state_proof
                            .verify(expected_hash, ap_hash, blob.as_ref())?;
                    }
                }
            }
        }
        let address_hash = account_address.crypto_hash();
        self.account_proof.verify(
            expected_root_hash,
            address_hash,
            self.account_state.as_ref(),
        )
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct StateWithProof {
    pub state: Option<Vec<u8>>,
    pub proof: StateProof,
}

impl StateWithProof {
    pub fn new(state: Option<Vec<u8>>, proof: StateProof) -> Self {
        Self { state, proof }
    }
}

pub trait ChainStateReader {
    /// Gets the state data for a single access path.
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>>;

    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof> {
        //TODO implements proof.
        self.get(access_path)
            .map(|state| StateWithProof::new(state, StateProof::default()))
    }

    /// Gets state data for a list of access paths.
    fn multi_get(&self, access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        access_paths
            .iter()
            .map(|access_path| self.get(access_path))
            .collect()
    }

    /// Gets account state
    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>>;

    /// VM needs this method to know whether the current state view is for genesis state creation.
    fn is_genesis(&self) -> bool;

    /// Gets current state root.
    fn state_root(&self) -> HashValue;

    fn dump(&self) -> Result<ChainStateSet>;
}

pub trait ChainStateWriter {
    /// Sets state at access_path.
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()>;

    /// Remove state at access_path
    fn remove(&self, access_path: &AccessPath) -> Result<()>;

    fn create_account(&self, account_address: AccountAddress) -> Result<()>;

    /// Apply dump result to ChainState
    fn apply(&self, state_set: ChainStateSet) -> Result<()>;

    fn commit(&self) -> Result<HashValue>;

    fn flush(&self) -> Result<()>;
}

/// `ChainState` is a trait that defines chain's global state.
pub trait ChainState: ChainStateReader + ChainStateWriter {}

/// `AccountStateReader` is a helper struct for read account state.
pub struct AccountStateReader<'a> {
    //TODO add a cache.
    reader: &'a dyn ChainStateReader,
}

impl<'a> AccountStateReader<'a> {
    pub fn new(reader: &'a dyn ChainStateReader) -> Self {
        Self { reader }
    }

    /// Get AccountResource by address
    pub fn get_account_resource(
        &self,
        address: &AccountAddress,
    ) -> Result<Option<AccountResource>> {
        self.reader
            .get(&AccessPath::new_for_account(*address))
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(AccountResource::make_from(bytes.as_slice())?)),
                None => Ok(None),
            })
    }

    /// Get starcoin account balance by address
    pub fn get_balance(&self, address: &AccountAddress) -> Result<Option<u64>> {
        Ok(self
            .reader
            .get(&AccessPath::new_for_balance(*address))
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(BalanceResource::make_from(bytes.as_slice())?)),
                None => Ok(None),
            })?
            .map(|resource| resource.coin()))
    }
}
