// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Result};
use merkle_tree::{blob::Blob, proof::SparseMerkleProof, RawKey};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_types::language_storage::StructTag;
use starcoin_types::state_set::AccountStateSet;
use starcoin_types::write_set::WriteSet;
use starcoin_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{AccountResource, BalanceResource},
    account_state::AccountState,
    state_set::ChainStateSet,
};
use starcoin_vm_types::account_config::{genesis_address, STC_TOKEN_CODE};
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::on_chain_resource::dao::{Proposal, ProposalAction};
use starcoin_vm_types::on_chain_resource::{
    Epoch, EpochData, EpochInfo, GlobalTimeOnChain, Treasury,
};
use starcoin_vm_types::sips::SIP;
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::token::token_info::TokenInfo;
use starcoin_vm_types::{
    move_resource::MoveResource, on_chain_config::OnChainConfig, state_view::StateView,
};
use std::convert::{TryFrom, TryInto};
use std::sync::Arc;

#[derive(Debug, Default, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct StateProof {
    pub account_state: Option<Blob>,
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
        let (account_address, data_path) = access_path.into_inner();
        match self.account_state.as_ref() {
            None => {
                ensure!(
                    access_resource_blob.is_none(),
                    "accessed resource should not exists"
                );
            }
            Some(s) => {
                let account_state = AccountState::try_from(s.as_ref())?;
                match account_state.storage_roots()[data_path.data_type().storage_index()] {
                    None => {
                        ensure!(
                            access_resource_blob.is_none(),
                            "accessed resource should not exists"
                        );
                    }
                    Some(expected_hash) => {
                        let blob = access_resource_blob.map(|data| Blob::from(data.to_vec()));
                        self.account_state_proof.verify(
                            expected_hash,
                            data_path.key_hash(),
                            blob.as_ref(),
                        )?;
                    }
                }
            }
        }
        self.account_proof.verify(
            expected_root_hash,
            account_address.key_hash(),
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

    pub fn get_state(&self) -> &Option<Vec<u8>> {
        &self.state
    }

    pub fn verify(&self, expect_root: HashValue, access_path: AccessPath) -> Result<()> {
        self.proof
            .verify(expect_root, access_path, self.state.as_deref())
    }
}

pub trait ChainStateReader: StateView {
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof>;

    /// Gets account state
    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>>;

    /// get whole state data of some account address.
    fn get_account_state_set(&self, address: &AccountAddress) -> Result<Option<AccountStateSet>>;

    fn exist_account(&self, address: &AccountAddress) -> Result<bool> {
        self.get_account_state(address).map(|state| state.is_some())
    }

    /// Gets current state root.
    fn state_root(&self) -> HashValue;

    fn dump(&self) -> Result<ChainStateSet>;
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

impl<T: ?Sized> StateReaderExt for T where T: StateView {}

pub trait StateReaderExt: StateView {
    /// Get AccountResource by address
    fn get_account_resource(&self, address: AccountAddress) -> Result<Option<AccountResource>> {
        self.get_resource::<AccountResource>(address)
    }

    /// Get Resource by type R
    fn get_resource<R>(&self, address: AccountAddress) -> Result<Option<R>>
    where
        R: MoveResource + DeserializeOwned,
    {
        let access_path = AccessPath::new(address, R::resource_path());
        self.get_resource_by_access_path(access_path)
    }

    fn get_resource_by_access_path<R>(&self, access_path: AccessPath) -> Result<Option<R>>
    where
        R: MoveResource + DeserializeOwned,
    {
        let r = self.get(&access_path).and_then(|state| match state {
            Some(state) => Ok(Some(bcs_ext::from_bytes::<R>(state.as_slice())?)),
            None => Ok(None),
        })?;
        Ok(r)
    }

    fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        self.get_account_resource(address)?
            .map(|resource| resource.sequence_number())
            .ok_or_else(|| format_err!("Can not find account by address:{}", address))
    }

    fn get_on_chain_config<C>(&self) -> Result<Option<C>>
    where
        C: OnChainConfig,
        Self: Sized,
    {
        C::fetch_config(self)
    }

    fn get_balance(&self, address: AccountAddress) -> Result<Option<u128>> {
        self.get_balance_by_token_code(address, STC_TOKEN_CODE.clone())
    }

    /// Get balance by address and coin type
    fn get_balance_by_type(
        &self,
        address: AccountAddress,
        type_tag: StructTag,
    ) -> Result<Option<u128>> {
        Ok(self
            .get(&AccessPath::new(
                address,
                BalanceResource::access_path_for(type_tag),
            ))
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(bcs_ext::from_bytes::<BalanceResource>(
                    bytes.as_slice(),
                )?)),
                None => Ok(None),
            })?
            .map(|resource| resource.token()))
    }

    fn get_balance_by_token_code(
        &self,
        address: AccountAddress,
        token_code: TokenCode,
    ) -> Result<Option<u128>> {
        self.get_balance_by_type(address, token_code.try_into()?)
    }

    fn get_epoch(&self) -> Result<Epoch> {
        self.get_resource::<Epoch>(genesis_address())?
            .ok_or_else(|| format_err!("Epoch is none."))
    }

    fn get_epoch_info(&self) -> Result<EpochInfo> {
        let epoch = self
            .get_resource::<Epoch>(genesis_address())?
            .ok_or_else(|| format_err!("Epoch is none."))?;

        let epoch_data = self
            .get_resource::<EpochData>(genesis_address())?
            .ok_or_else(|| format_err!("Epoch is none."))?;

        Ok(EpochInfo::new(epoch, epoch_data))
    }

    fn get_timestamp(&self) -> Result<GlobalTimeOnChain> {
        self.get_resource(genesis_address())?
            .ok_or_else(|| format_err!("Timestamp resource should exist."))
    }

    fn get_chain_id(&self) -> Result<ChainId> {
        self.get_resource::<ChainId>(genesis_address())?
            .ok_or_else(|| format_err!("ChainId resource should exist at genesis address. "))
    }

    fn get_code(&self, module_id: ModuleId) -> Result<Option<Vec<u8>>> {
        self.get(&AccessPath::from(&module_id))
    }

    /// Check the sip is activated. if the sip module exist, think it is activated.
    fn is_activated(&self, sip: SIP) -> Result<bool> {
        self.get_code(sip.module_id()).map(|code| code.is_some())
    }

    fn get_token_info(&self, token_code: TokenCode) -> Result<Option<TokenInfo>> {
        let type_tag = token_code.try_into()?;
        let access_path = TokenInfo::resource_path_for(type_tag);
        self.get_resource_by_access_path(access_path)
    }

    fn get_stc_info(&self) -> Result<Option<TokenInfo>> {
        self.get_token_info(STC_TOKEN_CODE.clone())
    }

    fn get_treasury(&self, token_code: TokenCode) -> Result<Option<Treasury>> {
        let access_path = Treasury::resource_path_for(token_code.try_into()?);
        self.get_resource_by_access_path(access_path)
    }

    fn get_stc_treasury(&self) -> Result<Option<Treasury>> {
        self.get_treasury(STC_TOKEN_CODE.clone())
    }

    fn get_proposal<A>(&self, token_code: TokenCode) -> Result<Option<Proposal<A>>>
    where
        A: ProposalAction + DeserializeOwned,
    {
        let access_path = Proposal::<A>::resource_path_for(token_code.try_into()?);
        self.get_resource_by_access_path(access_path)
    }

    fn get_stc_proposal<A>(&self) -> Result<Option<Proposal<A>>>
    where
        A: ProposalAction + DeserializeOwned,
    {
        self.get_proposal(STC_TOKEN_CODE.clone())
    }
}

/// `AccountStateReader` is a helper struct for read account state.
pub struct AccountStateReader<'a, Reader> {
    //TODO add a cache.
    reader: &'a Reader,
}

impl<'a, Reader> AccountStateReader<'a, Reader>
where
    Reader: ChainStateReader,
{
    pub fn new(reader: &'a Reader) -> Self {
        Self { reader }
    }

    /// Get AccountResource by address
    pub fn get_account_resource(
        &self,
        address: &AccountAddress,
    ) -> Result<Option<AccountResource>> {
        self.reader.get_account_resource(*address)
    }

    /// Get Resource by type
    pub fn get_resource<R>(&self, address: AccountAddress) -> Result<Option<R>>
    where
        R: MoveResource + DeserializeOwned,
    {
        self.reader.get_resource(address)
    }

    pub fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        self.reader.get_sequence_number(address)
    }

    pub fn get_on_chain_config<C>(&self) -> Result<Option<C>>
    where
        C: OnChainConfig,
    {
        self.reader.get_on_chain_config()
    }

    pub fn get_balance(&self, address: &AccountAddress) -> Result<Option<u128>> {
        self.reader.get_balance(*address)
    }

    /// Get balance by address and coin type
    pub fn get_balance_by_type(
        &self,
        address: &AccountAddress,
        type_tag: StructTag,
    ) -> Result<Option<u128>> {
        self.reader.get_balance_by_type(*address, type_tag)
    }

    pub fn get_balance_by_token_code(
        &self,
        address: &AccountAddress,
        token_code: TokenCode,
    ) -> Result<Option<u128>> {
        self.reader.get_balance_by_token_code(*address, token_code)
    }

    pub fn get_epoch(&self) -> Result<Epoch> {
        self.reader.get_epoch()
    }

    pub fn get_epoch_info(&self) -> Result<EpochInfo> {
        self.reader.get_epoch_info()
    }

    pub fn get_timestamp(&self) -> Result<GlobalTimeOnChain> {
        self.reader.get_timestamp()
    }

    pub fn get_chain_id(&self) -> Result<ChainId> {
        self.reader.get_chain_id()
    }
}
