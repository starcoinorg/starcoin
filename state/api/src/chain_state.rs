// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use merkle_tree::{blob::Blob, proof::SparseMerkleProof};
use serde::{Deserialize, Serialize};
use starcoin_crypto::{hash::PlainCryptoHash, HashValue};
use starcoin_types::write_set::{WriteOp, WriteSet};
use starcoin_types::{
    access_path::{self, AccessPath},
    account_address::AccountAddress,
    account_config::{AccountResource, BalanceResource},
    account_state::AccountState,
    language_storage::TypeTag,
    state_set::ChainStateSet,
};
use starcoin_vm_types::account_config::{stc_type_tag, type_tag_for_currency_code};
use starcoin_vm_types::on_chain_config::{ConfigStorage, OnChainConfig, RegisteredCurrencies};
use starcoin_vm_types::state_view::StateView;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;

#[derive(Debug, Default, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct StateProof {
    account_state: Option<Blob>,
    pub account_proof: SparseMerkleProof,
    pub account_state_proof: SparseMerkleProof,
}

impl StateProof {
    pub fn new(
        account_state: Option<Vec<u8>>,
        account_proof: SparseMerkleProof,
        account_state_proof: SparseMerkleProof,
    ) -> Self {
        Self {
            account_state: account_state.map(Blob::from),
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
        let (account_address, data_type, ap_hash) = access_path::into_inner(access_path)?;
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

pub trait ChainStateReader: StateView {
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof>;

    /// Gets account state
    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>>;

    /// Gets current state root.
    fn state_root(&self) -> HashValue;

    fn dump(&self) -> Result<ChainStateSet>;
}

impl ConfigStorage for &dyn ChainStateReader {
    fn fetch_config(&self, access_path: AccessPath) -> Option<Vec<u8>> {
        self.get(&access_path).ok().flatten()
    }
}

pub trait ChainStateWriter {
    /// Sets state at access_path.
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()>;

    /// Remove state at access_path
    fn remove(&self, access_path: &AccessPath) -> Result<()>;

    /// Apply dump result to ChainState
    fn apply(&self, state_set: ChainStateSet) -> Result<()>;

    //TODO support batch write.
    fn apply_write_set(&self, write_set: WriteSet) -> Result<()> {
        for (access_path, write_op) in write_set {
            match write_op {
                WriteOp::Value(blob) => {
                    self.set(&access_path, blob)?;
                }
                WriteOp::Deletion => {
                    self.remove(&access_path)?;
                }
            }
        }
        Ok(())
    }

    fn commit(&self) -> Result<HashValue>;

    fn flush(&self) -> Result<()>;
}

//This code is repeat with storage IntoSuper
//But can not share IntoSuper between different crate.
//only traits defined in the current crate can be implemented for a type parameter
pub trait IntoSuper<Super: ?Sized> {
    fn as_super(&self) -> &Super;
    fn as_super_mut(&mut self) -> &mut Super;
    fn into_super(self: Box<Self>) -> Box<Super>;
    fn into_super_arc(self: Arc<Self>) -> Arc<Super>;
}

/// `ChainState` is a trait that defines chain's global state.
pub trait ChainState:
    ChainStateReader
    + ChainStateWriter
    + StateView
    + IntoSuper<dyn StateView>
    + IntoSuper<dyn ChainStateReader>
    + IntoSuper<dyn ChainStateWriter>
{
}

impl<'a, T: 'a + ChainStateReader> IntoSuper<dyn ChainStateReader + 'a> for T {
    fn as_super(&self) -> &(dyn ChainStateReader + 'a) {
        self
    }
    fn as_super_mut(&mut self) -> &mut (dyn ChainStateReader + 'a) {
        self
    }
    fn into_super(self: Box<Self>) -> Box<dyn ChainStateReader + 'a> {
        self
    }
    fn into_super_arc(self: Arc<Self>) -> Arc<dyn ChainStateReader + 'a> {
        self
    }
}

impl<'a, T: 'a + ChainStateWriter> IntoSuper<dyn ChainStateWriter + 'a> for T {
    fn as_super(&self) -> &(dyn ChainStateWriter + 'a) {
        self
    }
    fn as_super_mut(&mut self) -> &mut (dyn ChainStateWriter + 'a) {
        self
    }
    fn into_super(self: Box<Self>) -> Box<dyn ChainStateWriter + 'a> {
        self
    }
    fn into_super_arc(self: Arc<Self>) -> Arc<dyn ChainStateWriter + 'a> {
        self
    }
}

impl<'a, T: 'a + StateView> IntoSuper<dyn StateView + 'a> for T {
    fn as_super(&self) -> &(dyn StateView + 'a) {
        self
    }
    fn as_super_mut(&mut self) -> &mut (dyn StateView + 'a) {
        self
    }
    fn into_super(self: Box<Self>) -> Box<dyn StateView + 'a> {
        self
    }
    fn into_super_arc(self: Arc<Self>) -> Arc<dyn StateView + 'a> {
        self
    }
}

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
                Some(bytes) => Ok(Some(scs::from_bytes::<AccountResource>(bytes.as_slice())?)),
                None => Ok(None),
            })
    }

    pub fn get_on_chain_config<C>(&self) -> Option<C>
    where
        C: OnChainConfig,
    {
        C::fetch_config(self.reader)
    }

    pub fn get_registered_currencies(&self) -> RegisteredCurrencies {
        self.get_on_chain_config()
            .expect("RegisteredCurrencies on chain config should exist.")
    }

    /// Get default coin balance by address
    pub fn get_balance(&self, address: &AccountAddress) -> Result<Option<u64>> {
        self.get_balance_by_type(address, &stc_type_tag())
    }

    /// Get balance by address and coin type
    pub fn get_balance_by_type(
        &self,
        address: &AccountAddress,
        type_tag: &TypeTag,
    ) -> Result<Option<u64>> {
        Ok(self
            .reader
            .get(&AccessPath::new(
                *address,
                BalanceResource::access_path_for(type_tag.clone()),
            ))
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(scs::from_bytes::<BalanceResource>(bytes.as_slice())?)),
                None => Ok(None),
            })?
            .map(|resource| resource.coin()))
    }

    /// Get all balance of account
    pub fn get_balances(&self, address: &AccountAddress) -> Result<HashMap<String, u64>> {
        let currencies = self.get_registered_currencies();
        let mut result = HashMap::new();
        //TODO batch get.
        for record in currencies.currency_codes() {
            let balance = self
                .get_balance_by_type(
                    &address,
                    &type_tag_for_currency_code(
                        Some(record.module_address),
                        record.currency_code.clone(),
                    ),
                )
                .ok()
                .flatten();
            if let Some(balance) = balance {
                result.insert(record.currency_code.as_str().to_owned(), balance);
            }
        }
        Ok(result)
    }
}
