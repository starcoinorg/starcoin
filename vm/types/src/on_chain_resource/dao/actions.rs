// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::on_chain_config::OnChainConfig;
use crate::on_chain_resource::dao::ProposalAction;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::{MoveResource, MoveStructType};
use serde::{Deserialize, Deserializer, Serialize};
use starcoin_crypto::HashValue;

/// A Rust representation of a UpgradeModule resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradeModule {
    pub module_address: AccountAddress,
    pub package_hash: HashValue,
    pub version: u64,
}

impl MoveStructType for UpgradeModule {
    const MODULE_NAME: &'static IdentStr = ident_str!("UpgradeModuleDaoProposal");
    const STRUCT_NAME: &'static IdentStr = ident_str!("UpgradeModule");
}

impl MoveResource for UpgradeModule {}

impl ProposalAction for UpgradeModule {}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradeModuleV2 {
    pub module_address: AccountAddress,
    pub package_hash: HashValue,
    pub version: u64,
    pub enforced: bool,
}

impl MoveStructType for UpgradeModuleV2 {
    const MODULE_NAME: &'static IdentStr = ident_str!("UpgradeModuleDaoProposal");
    const STRUCT_NAME: &'static IdentStr = ident_str!("UpgradeModuleV2");
}

impl MoveResource for UpgradeModuleV2 {}

impl ProposalAction for UpgradeModuleV2 {}

/// A Rust representation of a DaoConfigUpdate action.
#[derive(Debug, Serialize, Deserialize)]
pub struct DaoConfigUpdate {
    /// new voting delay setting.
    pub voting_delay: u64,
    /// new voting period setting.
    pub voting_period: u64,
    /// new voting quorum rate setting.
    pub voting_quorum_rate: u8,
    /// new min action delay setting.
    pub min_action_delay: u64,
}

impl MoveStructType for DaoConfigUpdate {
    const MODULE_NAME: &'static IdentStr = ident_str!("ModifyDaoConfigProposal");
    const STRUCT_NAME: &'static IdentStr = ident_str!("DaoConfigUpdate");
}

impl MoveResource for DaoConfigUpdate {}

impl ProposalAction for DaoConfigUpdate {}

/// A Rust representation of a OnChainConfigUpdate action.
#[derive(Debug, Serialize)]
pub struct OnChainConfigUpdate<C: OnChainConfig> {
    pub value: C,
}

impl<'de, C> Deserialize<'de> for OnChainConfigUpdate<C>
where
    C: OnChainConfig,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let value = C::deserialize(deserializer)?;
        Ok(Self { value })
    }
}

impl<C> MoveStructType for OnChainConfigUpdate<C>
where
    C: OnChainConfig,
{
    const MODULE_NAME: &'static IdentStr = ident_str!("OnChainConfigDao");
    const STRUCT_NAME: &'static IdentStr = ident_str!("OnChainConfigUpdate");
}

//TODO fixme
//impl<C> ProposalAction for OnChainConfigUpdate<C> where C: OnChainConfig {}

/// A Rust representation of a treasury WithdrawToken action.
#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawToken {
    /// the receiver of withdraw tokens.
    pub receiver: AccountAddress,
    /// how many tokens to mint.
    pub amount: u128,
    /// How long in milliseconds does it take for the token to be released
    pub period: u64,
}

impl MoveStructType for WithdrawToken {
    const MODULE_NAME: &'static IdentStr = ident_str!("TreasuryWithdrawDaoProposal");
    const STRUCT_NAME: &'static IdentStr = ident_str!("WithdrawToken");
}

impl MoveResource for WithdrawToken {}

impl ProposalAction for WithdrawToken {}
