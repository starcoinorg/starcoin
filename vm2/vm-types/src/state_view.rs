// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate defines [`trait StateView`](StateView).

use crate::state_store::state_key::{inner::StateKeyInner, StateKey};
use crate::state_store::StateView;
use crate::{
    account_config::{
        genesis_address,
        resources::{primary_store, ConcurrentFungibleBalanceResource, FungibleStoreResource},
        token_code::TokenCode,
        AccountResource, CoinStoreResource, ObjectGroupResource, TokenInfo, G_STC_TOKEN_CODE,
    },
    move_resource::MoveResource,
    on_chain_config::{Features, GlobalTimeOnChain, OnChainConfig},
    on_chain_resource::{
        dao::{Proposal, ProposalAction},
        BlockMetadata, ChainId, Epoch, EpochData, EpochInfo, Treasury,
    },
    sips::SIP,
};
use anyhow::{format_err, Result};
use bytes::Bytes;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
    move_resource::MoveStructType,
    vm_status::StatusCode,
};
use std::collections::BTreeMap;
use vm::errors::PartialVMError;

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

    fn get_resource_type_bytes<R>(&self, address: AccountAddress) -> Result<Bytes>
    where
        R: MoveResource,
    {
        self.get_state_value_bytes(&StateKey::resource_typed::<R>(&address)?)?
            .ok_or_else(|| {
                format_err!(
                    "Resource {:?} {:?} not exists at address:{}",
                    R::module_identifier(),
                    R::struct_identifier(),
                    address
                )
            })
    }

    /// Get Resource by type R
    fn get_resource_type<R>(&self, address: AccountAddress) -> Result<R>
    where
        R: MoveResource,
    {
        let rsrc_bytes = self.get_resource_type_bytes::<R>(address)?;
        let rsrc = bcs_ext::from_bytes::<R>(&rsrc_bytes)?;
        Ok(rsrc)
    }

    fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        Ok(self.get_account_resource(address)?.sequence_number())
    }

    fn get_on_chain_config<T>(&self) -> Option<T>
    where
        T: OnChainConfig,
        Self: Sized,
    {
        T::fetch_config(self)
    }

    /// Get balance by address and coin type
    fn get_balance_by_type(&self, address: AccountAddress, type_tag: StructTag) -> Result<u128> {
        let coin_store_bytes = self.get_state_value_bytes(&StateKey::resource(
            &address,
            &CoinStoreResource::struct_tag_for_token(type_tag.clone()),
        )?)?;

        let mut total_balance: u128 = match coin_store_bytes {
            Some(bytes) => bcs_ext::from_bytes::<CoinStoreResource>(&bytes)?.coin() as u128,
            None => 0,
        };

        // Read primary fungible store from user
        let primary_fungible_store_address =
            primary_store(&address, &type_tag.to_canonical_string())?;
        let split_enabled = match StateKey::on_chain_config::<Features>() {
            Ok(state_key) => self
                .get_state_value_bytes(&state_key)?
                .map(|bytes| Features::deserialize_into_config(&bytes))
                .transpose()?
                .map(|features| features.is_resource_groups_split_in_vm_change_set_enabled())
                .unwrap_or(false),
            Err(_) => false,
        };

        let tag_bytes = self.get_resource_group_struct_tag_bytes_with_flag(
            &address,
            &StateKey::resource_group(
                &primary_fungible_store_address,
                &ObjectGroupResource::struct_tag(),
            ),
            &FungibleStoreResource::struct_tag(),
            split_enabled,
        )?;

        let concurrent_balance_bytes = self.get_resource_group_struct_tag_bytes_with_flag(
            &address,
            &StateKey::resource_group(
                &primary_fungible_store_address,
                &ObjectGroupResource::struct_tag(),
            ),
            &ConcurrentFungibleBalanceResource::struct_tag(),
            split_enabled,
        )?;

        if let Some(bytes) = concurrent_balance_bytes {
            let concurrent_balance =
                bcs_ext::from_bytes::<ConcurrentFungibleBalanceResource>(&bytes)?;
            total_balance += concurrent_balance.balance() as u128;
        } else if let Some(bytes) = tag_bytes {
            let fungible_store = bcs_ext::from_bytes::<FungibleStoreResource>(&bytes)?;
            total_balance += fungible_store.balance() as u128;
        }

        Ok(total_balance)
    }

    fn get_resource_group_struct_tag_bytes(
        &self,
        address: &AccountAddress,
        group_key: &StateKey,
        struct_tag: &StructTag,
    ) -> Result<Option<Bytes>> {
        let split_enabled = match StateKey::on_chain_config::<Features>() {
            Ok(state_key) => self
                .get_state_value_bytes(&state_key)?
                .map(|bytes| Features::deserialize_into_config(&bytes))
                .transpose()?
                .map(|features| features.is_resource_groups_split_in_vm_change_set_enabled())
                .unwrap_or(false),
            Err(_) => false,
        };
        self.get_resource_group_struct_tag_bytes_with_flag(
            address,
            group_key,
            struct_tag,
            split_enabled,
        )
    }

    fn get_resource_group_struct_tag_bytes_with_flag(
        &self,
        address: &AccountAddress,
        group_key: &StateKey,
        struct_tag: &StructTag,
        split_enabled: bool,
    ) -> Result<Option<Bytes>> {
        let group_address = match group_key.inner() {
            StateKeyInner::AccessPath(access_path) => &access_path.address,
            _ => address,
        };
        let member_key = StateKey::resource_group(group_address, struct_tag);
        if split_enabled {
            if let Some(bytes) = self.get_state_value_bytes(&member_key)? {
                return Ok(Some(bytes));
            }
        }

        let group_data = match self.get_state_value_bytes(group_key)? {
            Some(data) => data,
            None => {
                // When resource groups are split in the VM change set, members are stored
                // directly under their own resource group keys instead of a single map blob.
                return Ok(self.get_state_value_bytes(&member_key)?);
            }
        };

        let group_data_map: BTreeMap<StructTag, Bytes> = bcs::from_bytes::<
            BTreeMap<StructTag, Bytes>,
        >(&group_data)
        .map_err(|e| {
            PartialVMError::new(StatusCode::UNEXPECTED_DESERIALIZATION_ERROR).with_message(format!(
                "Failed to deserialize the resource group at {:? }: {:?}",
                group_key, e
            ))
        })?;

        if let Some(bytes) = group_data_map.get(struct_tag) {
            return Ok(Some(bytes.clone()));
        }

        // If the group blob doesn't contain the member, fall back to a direct lookup.
        // This covers the split resource group storage layout.
        Ok(self.get_state_value_bytes(&member_key)?)
    }

    fn get_epoch(&self) -> Result<Epoch> {
        self.get_resource_type::<Epoch>(genesis_address())
    }

    fn get_epoch_info(&self) -> Result<EpochInfo> {
        let epoch = self.get_resource_type::<Epoch>(genesis_address())?;

        let epoch_data = self.get_resource_type::<EpochData>(genesis_address())?;

        Ok(EpochInfo::new(epoch, epoch_data))
    }

    fn get_timestamp(&self) -> Result<GlobalTimeOnChain> {
        self.get_resource_type::<GlobalTimeOnChain>(genesis_address())
    }

    fn get_chain_id(&self) -> Result<ChainId> {
        self.get_resource_type::<ChainId>(genesis_address())
    }

    // Get BlockMetadata on chain (stdlib version <= 11)
    fn get_block_metadata(&self) -> Result<BlockMetadata> {
        self.get_resource_type::<BlockMetadata>(genesis_address())
    }

    fn get_code(&self, module_id: ModuleId) -> Result<Bytes> {
        self.get_state_value_bytes(&StateKey::module_id(&module_id))?
            .ok_or_else(|| format_err!("Can not find code by module_id:{}", module_id))
    }

    /// Check the sip is activated. if the sip module exist, think it is activated.
    fn is_activated(&self, sip: SIP) -> Result<bool> {
        self.get_code(sip.module_id()).map(|code| !code.is_empty())
    }

    fn get_token_info(&self, token_code: TokenCode) -> Result<TokenInfo> {
        let type_tag: StructTag = token_code.clone().try_into()?;
        let rsrc_bytes =
            self.get_resource(token_code.address, &TokenInfo::struct_tag_for(type_tag))?;
        let rsrc = bcs_ext::from_bytes::<TokenInfo>(&rsrc_bytes)?;
        Ok(rsrc)
    }

    fn get_stc_info(&self) -> Result<TokenInfo> {
        self.get_token_info(G_STC_TOKEN_CODE.clone())
    }

    fn get_treasury(&self, token_code: TokenCode) -> Result<Treasury> {
        let type_tag: StructTag = token_code.clone().try_into()?;
        let rsrc_bytes =
            self.get_resource(token_code.address, &Treasury::struct_tag_for(type_tag))?;
        let rsrc = bcs_ext::from_bytes::<Treasury>(&rsrc_bytes)?;
        Ok(rsrc)
    }

    fn get_stc_treasury(&self) -> Result<Treasury> {
        self.get_treasury(G_STC_TOKEN_CODE.clone())
    }

    //TODO update to new DAOSpace proposal
    fn get_proposal<A>(&self, token_code: TokenCode) -> Result<Proposal<A>>
    where
        A: ProposalAction,
    {
        let type_tag: StructTag = token_code.clone().try_into()?;
        let rsrc_bytes =
            self.get_resource(token_code.address, &Proposal::<A>::struct_tag_for(type_tag))?;
        let rsrc = bcs_ext::from_bytes::<Proposal<A>>(&rsrc_bytes)?;
        Ok(rsrc)
    }

    fn get_stc_proposal<A>(&self) -> Result<Proposal<A>>
    where
        A: ProposalAction,
    {
        self.get_proposal(G_STC_TOKEN_CODE.clone())
    }
}
