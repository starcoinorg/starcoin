// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Result};
use merkle_tree::{blob::Blob, proof::SparseMerkleProof};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{hash::PlainCryptoHash, HashValue};
use starcoin_types::write_set::WriteSet;
use starcoin_types::{
    access_path::{self, AccessPath},
    account_address::AccountAddress,
    account_config::{AccountResource, BalanceResource},
    account_state::AccountState,
    language_storage::TypeTag,
    state_set::ChainStateSet,
};
use starcoin_vm_types::account_config::{genesis_address, STC_TOKEN_CODE};
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::on_chain_resource::{Epoch, EpochData, EpochInfo, GlobalTimeOnChain};
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::{
    move_resource::MoveResource,
    on_chain_config::{ConfigStorage, OnChainConfig},
    state_view::StateView,
};
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

    fn exist_account(&self, address: &AccountAddress) -> Result<bool> {
        self.get_account_state(address).map(|state| state.is_some())
    }

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
    fn apply_write_set(&self, write_set: WriteSet) -> Result<()>;

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

    //TODO change to pass Address copy not ref.
    /// Get AccountResource by address
    pub fn get_account_resource(
        &self,
        address: &AccountAddress,
    ) -> Result<Option<AccountResource>> {
        self.get_resource::<AccountResource>(*address)
    }

    /// Get Resource by type
    pub fn get_resource<R>(&self, address: AccountAddress) -> Result<Option<R>>
    where
        R: MoveResource + DeserializeOwned,
    {
        let access_path = AccessPath::new(address, R::resource_path());
        let r = self
            .reader
            .get(&access_path)
            .and_then(|state| match state {
                Some(state) => Ok(Some(scs::from_bytes::<R>(state.as_slice())?)),
                None => Ok(None),
            })?;
        Ok(r)
    }

    pub fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        self.get_account_resource(&address)?
            .map(|resource| resource.sequence_number())
            .ok_or_else(|| format_err!("Can not find account by address:{}", address))
    }

    pub fn get_on_chain_config<C>(&self) -> Result<Option<C>>
    where
        C: OnChainConfig,
    {
        C::fetch_config(self.reader)
    }

    pub fn get_balance(&self, address: &AccountAddress) -> Result<Option<u128>> {
        self.get_balance_by_token_code(address, STC_TOKEN_CODE.clone())
    }

    /// Get balance by address and coin type
    pub fn get_balance_by_type(
        &self,
        address: &AccountAddress,
        type_tag: TypeTag,
    ) -> Result<Option<u128>> {
        Ok(self
            .reader
            .get(&AccessPath::new(
                *address,
                BalanceResource::access_path_for(type_tag),
            ))
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(scs::from_bytes::<BalanceResource>(bytes.as_slice())?)),
                None => Ok(None),
            })?
            .map(|resource| resource.token()))
    }

    pub fn get_balance_by_token_code(
        &self,
        address: &AccountAddress,
        token_code: TokenCode,
    ) -> Result<Option<u128>> {
        self.get_balance_by_type(address, token_code.into())
    }

    pub fn get_epoch_info(&self) -> Result<EpochInfo> {
        let epoch = self
            .get_resource::<Epoch>(genesis_address())?
            .ok_or_else(|| format_err!("Epoch is none."))?;

        let epoch_data = self
            .get_resource::<EpochData>(genesis_address())?
            .ok_or_else(|| format_err!("Epoch is none."))?;

        Ok(EpochInfo::new(epoch, epoch_data))
    }

    pub fn get_timestamp(&self) -> Result<GlobalTimeOnChain> {
        self.get_resource(genesis_address())?
            .ok_or_else(|| format_err!("Timestamp resource should exist."))
    }

    pub fn get_chain_id(&self) -> Result<ChainId> {
        self.get_resource::<ChainId>(genesis_address())?
            .ok_or_else(|| format_err!("ChainId resource should exist at genesis address. "))
    }
}
