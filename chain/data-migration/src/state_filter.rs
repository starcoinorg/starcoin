// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bcs_ext;
use log::debug;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet, StateSet};
use starcoin_vm_types::{account_config::AccountResource, language_storage::StructTag};

/// Resource paths that should be completely filtered during migration
pub const FILTERED_RESOURCE_PATHS: &[&str] = &[
    // ChainId resource - contains network identifier
    "0x00000000000000000000000000000001::ChainId::ChainId",
    // BlockMetadata resource - contains block-specific information
    "0x00000000000000000000000000000001::Block::BlockMetadata",
];

/// Resource paths that should be modified (not filtered) during migration
pub const MODIFIED_RESOURCE_PATHS: &[&str] = &[
    // Account resource - sequence_number should be reset to 0
    "0x00000000000000000000000000000001::Account::Account",
];

/// Get the list of filtered resource paths for logging/debugging
pub fn get_filtered_resource_paths() -> &'static [&'static str] {
    FILTERED_RESOURCE_PATHS
}

/// Check if a resource should be filtered based on its struct tag
pub fn should_filter_resource(struct_tag: &StructTag) -> bool {
    let struct_tag_str = format!(
        "{}::{}::{}",
        struct_tag.address, struct_tag.module, struct_tag.name
    );
    FILTERED_RESOURCE_PATHS.contains(&struct_tag_str.as_str())
}

/// Check if a resource should be modified (not filtered) based on its struct tag
pub fn should_modify_resource(struct_tag: &StructTag) -> bool {
    let struct_tag_str = format!(
        "{}::{}::{}",
        struct_tag.address, struct_tag.module, struct_tag.name
    );
    MODIFIED_RESOURCE_PATHS.contains(&struct_tag_str.as_str())
}

/// Modify AccountResource by resetting sequence_number to 0
/// Uses the clone_with_zero_seq_number method to properly handle all fields
fn modify_account_resource(blob: &[u8]) -> anyhow::Result<Vec<u8>> {
    let resource = bcs_ext::from_bytes::<AccountResource>(blob)?;
    Ok(bcs_ext::to_bytes(&resource.clone_with_zero_seq_number())?)
}

/// Filter ChainStateSet by removing filtered resources and modifying specific resources
pub fn filter_chain_state_set(chain_state_set: ChainStateSet) -> anyhow::Result<ChainStateSet> {
    let mut filtered_state_set_vec = Vec::new();
    for (address, account_state_set) in chain_state_set.state_sets() {
        debug!("filtered_state_sets | address: {:?}", address);
        let mut filtered_resource_state_set = vec![];

        if let Some(resource_set) = account_state_set.resource_set() {
            for (key, blob) in resource_set.iter() {
                let struct_tag = bcs_ext::from_bytes::<StructTag>(key)?;
                if should_filter_resource(&struct_tag) {
                    debug!("Filtering resource {} for address {}", struct_tag, address);
                    continue;
                }

                // For now, keep all resources as-is (Account modification is commented out)
                let filtered_blob = if should_modify_resource(&struct_tag) {
                    modify_account_resource(blob)?
                } else {
                    blob.clone()
                };
                filtered_resource_state_set.push((key.clone(), filtered_blob));
            }
        }

        let new_account_state_set = AccountStateSet::new(vec![
            account_state_set.code_set().cloned(),
            Some(StateSet::new(filtered_resource_state_set)),
        ]);

        filtered_state_set_vec.push((*address, new_account_state_set));
    }
    Ok(ChainStateSet::new(filtered_state_set_vec))
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_vm_types::account_config::{
        genesis_address, token_code::TokenCode, BalanceResource, STC_TOKEN_CODE_STR,
    };
    use starcoin_vm_types::genesis_config::ChainId;
    use starcoin_vm_types::move_resource::MoveResource;
    use std::str::FromStr;

    #[test]
    fn test_should_filter_resource() {
        // Test ChainId resource
        let chain_id_tag = ChainId::struct_tag();
        println!(
            "ChainId struct_tag: {}, should be filtered: {}",
            chain_id_tag,
            should_filter_resource(&chain_id_tag)
        );
        assert!(should_filter_resource(&chain_id_tag));

        // Test BlockMetadata resource (on-chain resource)
        let block_metadata_tag = starcoin_vm_types::on_chain_resource::BlockMetadata::struct_tag();
        println!(
            "BlockMetadata struct_tag: {}, should be filtered: {}",
            block_metadata_tag,
            should_filter_resource(&block_metadata_tag)
        );
        assert!(should_filter_resource(&block_metadata_tag));

        // Test Account resource
        let account_tag = starcoin_vm_types::account_config::AccountResource::struct_tag();
        println!(
            "Account struct_tag: {}, should be filtered: {}, should be modified: {}",
            account_tag,
            should_filter_resource(&account_tag),
            should_modify_resource(&account_tag)
        );
        assert!(!should_filter_resource(&account_tag));
        assert!(should_modify_resource(&account_tag));

        // Test a resource that should not be filtered
        let balance_tag = BalanceResource::struct_tag_for_token(
            TokenCode::from_str(STC_TOKEN_CODE_STR)
                .unwrap()
                .try_into()
                .unwrap(),
        );
        println!(
            "Balance struct_tag: {}, should be filtered: {}",
            balance_tag,
            should_filter_resource(&balance_tag)
        );
        assert!(!should_filter_resource(&balance_tag));
    }

    #[test]
    fn test_filter_chain_state_set() {
        // Create a simple ChainStateSet with some test data
        let mut test_resources = Vec::new();

        // Add a resource that should be filtered (ChainId)
        let chain_id_tag = ChainId::struct_tag();
        let chain_id_bytes = bcs_ext::to_bytes(&ChainId::test()).unwrap();
        test_resources.push((bcs_ext::to_bytes(&chain_id_tag).unwrap(), chain_id_bytes));

        // Add a resource that should not be filtered (Balance)
        let balance_tag = BalanceResource::struct_tag_for_token(
            TokenCode::from_str(STC_TOKEN_CODE_STR)
                .unwrap()
                .try_into()
                .unwrap(),
        );
        let balance_resource = BalanceResource::new(1000u128);
        let balance_bytes = bcs_ext::to_bytes(&balance_resource).unwrap();
        test_resources.push((bcs_ext::to_bytes(&balance_tag).unwrap(), balance_bytes));

        let state_set = StateSet::new(test_resources);
        let account_state_set = AccountStateSet::new(vec![Some(state_set), None]);

        let chain_state_set = ChainStateSet::new(vec![(genesis_address(), account_state_set)]);

        // Filter the ChainStateSet
        let filtered_set = filter_chain_state_set(chain_state_set);

        // Verify that the filtered set contains only non-filtered resources
        for (_address, account_state_set) in filtered_set.unwrap().state_sets() {
            if let Some(resource_set) = account_state_set.resource_set() {
                for (key, _value) in resource_set.iter() {
                    if let Ok(struct_tag) = bcs_ext::from_bytes::<StructTag>(key) {
                        // Should not contain filtered resources
                        assert!(
                            !should_filter_resource(&struct_tag),
                            "Filtered set should not contain filtered resource: {}",
                            struct_tag
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_get_filtered_resource_paths() {
        let paths = get_filtered_resource_paths();
        assert!(!paths.is_empty());
        assert!(paths.contains(&"0x00000000000000000000000000000001::ChainId::ChainId"));
        assert!(paths.contains(&"0x00000000000000000000000000000001::Block::BlockMetadata"));
        // Account is now in MODIFIED_RESOURCE_PATHS, not FILTERED_RESOURCE_PATHS
        assert!(!paths.contains(&"0x00000000000000000000000000000001::Account::Account"));
    }
}
