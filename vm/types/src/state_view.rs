// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate defines [`trait StateView`](StateView).

use crate::state_store::state_key::StateKey;
use crate::{
    access_path::AccessPath,
    account_config::{
        genesis_address, token_code::TokenCode, AccountResource, BalanceResource, TokenInfo,
        G_STC_TOKEN_CODE,
    },
    genesis_config::ChainId,
    move_resource::MoveResource,
    on_chain_config::{GlobalTimeOnChain, OnChainConfig, ConfigStorage},
    on_chain_resource::{
        dao::{Proposal, ProposalAction},
        BlockMetadata, Epoch, EpochData, EpochInfo, Treasury,
    },
    sips::SIP,
};
use anyhow::{format_err, Result};
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
};
use serde::de::DeserializeOwned;
use std::ops::Deref;

/// `StateView` is a trait that defines a read-only snapshot of the global state. It is passed to
/// the VM for transaction execution, during which the VM is guaranteed to read anything at the
/// given state.
pub trait StateView: Sync {
    /// Gets the state value for a given state key.
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<Vec<u8>>>;

    /// VM needs this method to know whether the current state view is for genesis state creation.
    /// Currently TransactionPayload::WriteSet is only valid for genesis state creation.
    fn is_genesis(&self) -> bool;
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
        let r = self
            .get_state_value(&StateKey::AccessPath(access_path))
            .and_then(|state| match state {
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
        self.get_balance_by_token_code(address, G_STC_TOKEN_CODE.clone())
    }

    /// Get balance by address and coin type
    fn get_balance_by_type(
        &self,
        address: AccountAddress,
        type_tag: StructTag,
    ) -> Result<Option<u128>> {
        Ok(self
            .get_state_value(&StateKey::AccessPath(AccessPath::new(
                address,
                BalanceResource::access_path_for(type_tag),
            )))
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

    // Get latest BlockMetadata on chain
    fn get_block_metadata(&self) -> Result<BlockMetadata> {
        self.get_resource::<BlockMetadata>(genesis_address())?
            .ok_or_else(|| format_err!("BlockMetadata resource should exist at genesis address. "))
    }

    fn get_code(&self, module_id: ModuleId) -> Result<Option<Vec<u8>>> {
        self.get_state_value(&StateKey::AccessPath(AccessPath::from(&module_id)))
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
        self.get_token_info(G_STC_TOKEN_CODE.clone())
    }

    fn get_treasury(&self, token_code: TokenCode) -> Result<Option<Treasury>> {
        let access_path = Treasury::resource_path_for(token_code.try_into()?);
        self.get_resource_by_access_path(access_path)
    }

    fn get_stc_treasury(&self) -> Result<Option<Treasury>> {
        self.get_treasury(G_STC_TOKEN_CODE.clone())
    }

    //TOODO update to new DAOSpace proposal
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
        self.get_proposal(G_STC_TOKEN_CODE.clone())
    }
}

/// XXX FIXME YSG, why has conflict
impl<R, S> StateView for R
where
    R: Deref<Target = S> + Sync,
    S: StateView,
{
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<Vec<u8>>> {
        self.deref().get_state_value(state_key)
    }

    fn is_genesis(&self) -> bool {
        self.deref().is_genesis()
    }
}