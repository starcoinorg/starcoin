// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate defines [`trait StateView`](StateView).

use crate::state_store::state_key::StateKey;
use crate::state_store::StateView;
use crate::{
    account_config::{
        genesis_address, token_code::TokenCode, AccountResource, BalanceResource, TokenInfo,
        G_STC_TOKEN_CODE,
    },
    genesis_config::ChainId,
    move_resource::MoveResource,
    on_chain_config::{GlobalTimeOnChain, OnChainConfig},
    on_chain_resource::{
        dao::{Proposal, ProposalAction},
        BlockMetadata, BlockMetadataV2, Epoch, EpochData, EpochInfo, Treasury,
    },
    sips::SIP,
};
use anyhow::{format_err, Result};
use bytes::Bytes;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
};
use serde::de::DeserializeOwned;

impl<T: ?Sized> StateReaderExt for T where T: StateView {}

pub trait StateReaderExt: StateView {
    /// Get AccountResource by address
    fn get_account_resource(&self, address: AccountAddress) -> Result<AccountResource> {
        self.get_resource_type::<AccountResource>(address)
    }

    /// Get Resource by StructTag
    fn get_resource(&self, address: AccountAddress, struct_tag: &StructTag) -> Result<Bytes> {
        let rsrc_bytes = self
            .get_state_value_bytes(&StateKey::resource(&address, struct_tag)?)?
            .ok_or_else(|| {
                format_err!(
                    "Resource {:?} not exists at address:{}",
                    struct_tag,
                    address
                )
            })?;
        Ok(rsrc_bytes)
    }

    /// Get Resource by type R
    fn get_resource_type<R>(&self, address: AccountAddress) -> Result<R>
    where
        R: MoveResource + DeserializeOwned,
    {
        let rsrc_bytes = self
            .get_state_value_bytes(&StateKey::resource_typed::<R>(&address)?)?
            .ok_or_else(|| {
                format_err!(
                    "Resource {:?} {:?} not exists at address:{}",
                    R::module_identifier(),
                    R::struct_identifier(),
                    address
                )
            })?;
        let rsrc = bcs_ext::from_bytes::<R>(&rsrc_bytes)?;
        Ok(rsrc)
    }

    fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        self.get_account_resource(address)?
            .map(|resource| resource.sequence_number())
            .ok_or_else(|| format_err!("Can not find account by address:{}", address))
    }

    fn get_on_chain_config<T>(&self) -> Option<T>
    where
        T: OnChainConfig,
        Self: Sized,
    {
        T::fetch_config(self)
    }

    fn get_balance(&self, address: AccountAddress) -> Result<u128> {
        self.get_balance_by_token_code(address, G_STC_TOKEN_CODE.clone())
    }

    /// Get balance by address and coin type
    fn get_balance_by_type(&self, address: AccountAddress, type_tag: StructTag) -> Result<u128> {
        let rsrc_bytes = self
            .get_state_value_bytes(&StateKey::resource(
                &address,
                &BalanceResource::struct_tag_for_token(type_tag),
            )?)?
            .ok_or_else(|| {
                format_err!(
                    "BalanceResource not exists at address:{} for type tag:{}",
                    address,
                    type_tag
                )
            })?;
        let rsrc = bcs_ext::from_bytes::<BalanceResource>(&rsrc_bytes)?;
        Ok(rsrc.token())
    }

    fn get_balance_by_token_code(
        &self,
        address: AccountAddress,
        token_code: TokenCode,
    ) -> Result<u128> {
        self.get_balance_by_type(address, token_code.try_into()?)
    }

    fn get_epoch(&self) -> Result<Epoch> {
        self.get_resource_type::<Epoch>(genesis_address())?
            .ok_or_else(|| format_err!("Epoch is none."))
    }

    fn get_epoch_info(&self) -> Result<EpochInfo> {
        let epoch = self.get_resource_type::<Epoch>(genesis_address())?;

        let epoch_data = self.get_resource_type::<EpochData>(genesis_address())?;

        Ok(EpochInfo::new(epoch, epoch_data))
    }

    fn get_timestamp(&self) -> Result<GlobalTimeOnChain> {
        self.get_resource_type(genesis_address())?
            .ok_or_else(|| format_err!("Timestamp resource should exist."))
    }

    fn get_chain_id(&self) -> Result<ChainId> {
        self.get_resource_type::<ChainId>(genesis_address())
    }

    // Get BlockMetadata on chain (stdlib version <= 11)
    fn get_block_metadata(&self) -> Result<BlockMetadata> {
        self.get_resource_type::<BlockMetadata>(genesis_address())
    }

    // Get latest BlockMetadataV2 on chain, since stdlib version(12)
    fn get_block_metadata_v2(&self) -> Result<BlockMetadataV2> {
        self.get_resource_type::<BlockMetadataV2>(genesis_address())
    }

    fn get_code(&self, module_id: ModuleId) -> Result<Bytes> {
        self.get_state_value_bytes(&StateKey::module_id(&module_id)?)?
            .ok_or_else(|| format_err!("Can not find code by module_id:{}", module_id))
    }

    /// Check the sip is activated. if the sip module exist, think it is activated.
    fn is_activated(&self, sip: SIP) -> Result<bool> {
        self.get_code(sip.module_id()).map(|code| code.is_some())
    }

    fn get_token_info(&self, token_code: TokenCode) -> Result<TokenInfo> {
        let type_tag: StructTag = token_code.try_into()?;
        let rsrc_bytes = self.get_resource(
            token_code.address.clone(),
            &TokenInfo::struct_tag_for_token(type_tag),
        )?;
        let rsrc = bcs_ext::from_bytes::<TokenInfo>(&rsrc_bytes)?;
        Ok(rsrc)
    }

    fn get_stc_info(&self) -> Result<TokenInfo> {
        self.get_token_info(G_STC_TOKEN_CODE.clone())
    }

    fn get_treasury(&self, token_code: TokenCode) -> Result<Treasury> {
        let type_tag: StructTag = token_code.try_into()?;
        let rsrc_bytes = self.get_resource(
            token_code.address.clone(),
            &Treasury::struct_tag_for_token(type_tag),
        )?;
        let rsrc = bcs_ext::from_bytes::<Treasury>(&rsrc_bytes)?;
        Ok(rsrc)
    }

    fn get_stc_treasury(&self) -> Result<Treasury> {
        self.get_treasury(G_STC_TOKEN_CODE.clone())
    }

    //TODO update to new DAOSpace proposal
    fn get_proposal<A>(&self, token_code: TokenCode) -> Result<Proposal<A>>
    where
        A: ProposalAction + DeserializeOwned,
    {
        let type_tag: StructTag = token_code.try_into()?;
        let rsrc_bytes = self.get_resource(
            token_code.address.clone(),
            &Proposal::<A>::struct_tag_for_token(type_tag),
        )?;
        let rsrc = bcs_ext::from_bytes::<Proposal<A>>(&rsrc_bytes)?;
        Ok(rsrc)
    }

    fn get_stc_proposal<A>(&self) -> Result<Proposal<A>>
    where
        A: ProposalAction + DeserializeOwned,
    {
        self.get_proposal(G_STC_TOKEN_CODE.clone())
    }
}
