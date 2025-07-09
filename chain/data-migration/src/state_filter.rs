// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bcs_ext;
use log::debug;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet, StateSet};
use starcoin_vm_types::language_storage::StructTag;

/// Resource paths that should be filtered during migration
pub const FILTERED_RESOURCE_PATHS: &[&str] = &[
    // ChainId resource - contains network identifier
    "0x00000000000000000000000000000001::ChainId::ChainId",
    // BlockMetadata resource - contains block-specific information
    "0x00000000000000000000000000000001::Block::BlockMetadata",
    // Account sequence_number - should be reset for new network
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

/// Filter ChainStateSet by removing filtered resources
pub fn filter_chain_state_set(chain_state_set: ChainStateSet) -> ChainStateSet {
    let mut filtered_state_sets = Vec::new();
    for (address, account_state_set) in chain_state_set.state_sets() {
        let mut filtered_account_state_set = account_state_set.clone();
        // Filter resources if they exist
        if let Some(resource_set) = account_state_set.resource_set() {
            let mut filtered_resources = Vec::new();
            for (key, value) in resource_set.iter() {
                if let Ok(struct_tag) = bcs_ext::from_bytes::<StructTag>(key) {
                    if !should_filter_resource(&struct_tag) {
                        filtered_resources.push((key.clone(), value.clone()));
                    } else {
                        debug!("Filtering resource {} for address {}", struct_tag, address);
                    }
                } else {
                    // If we can't parse the struct tag, keep it to be safe
                    filtered_resources.push((key.clone(), value.clone()));
                }
            }
            // Create new StateSet with filtered resources
            let filtered_state_set = StateSet::new(filtered_resources);
            filtered_account_state_set = AccountStateSet::new(vec![
                Some(filtered_state_set),              // Resource set
                account_state_set.code_set().cloned(), // Code set
            ]);
        }
        filtered_state_sets.push((*address, filtered_account_state_set));
    }
    ChainStateSet::new(filtered_state_sets)
}

/// Add a new resource path to the filter list
/// Note: This is a placeholder for future extensibility
pub fn add_filtered_resource_path(_path: &'static str) {
    // Note: This is a placeholder for future extensibility
    // In a real implementation, you might want to use a mutable static or configuration
    debug!("Adding new filtered resource path: {}", _path);
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
        println!("ChainId struct_tag: {}", chain_id_tag);
        println!(
            "ChainId should be filtered: {}",
            should_filter_resource(&chain_id_tag)
        );
        assert!(should_filter_resource(&chain_id_tag));

        // Test BlockMetadata resource (on-chain resource)
        let block_metadata_tag = starcoin_vm_types::on_chain_resource::BlockMetadata::struct_tag();
        println!("BlockMetadata struct_tag: {}", block_metadata_tag);
        println!(
            "BlockMetadata should be filtered: {}",
            should_filter_resource(&block_metadata_tag)
        );
        assert!(should_filter_resource(&block_metadata_tag));

        // Test Account resource
        let account_tag = starcoin_vm_types::account_config::AccountResource::struct_tag();
        println!("Account struct_tag: {}", account_tag);
        println!(
            "Account should be filtered: {}",
            should_filter_resource(&account_tag)
        );
        assert!(should_filter_resource(&account_tag));

        // Test a resource that should not be filtered
        let balance_tag = BalanceResource::struct_tag_for_token(
            TokenCode::from_str(STC_TOKEN_CODE_STR)
                .unwrap()
                .try_into()
                .unwrap(),
        );
        println!("Balance struct_tag: {}", balance_tag);
        println!(
            "Balance should be filtered: {}",
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

        let state_set = starcoin_types::state_set::StateSet::new(test_resources);
        let account_state_set = starcoin_types::state_set::AccountStateSet::new(vec![
            Some(state_set), // Resource set
            None,            // Code set
        ]);

        let chain_state_set = ChainStateSet::new(vec![(genesis_address(), account_state_set)]);

        // Filter the ChainStateSet
        let filtered_set = filter_chain_state_set(chain_state_set);

        // Verify that the filtered set contains only non-filtered resources
        for (_address, account_state_set) in filtered_set.state_sets() {
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
        assert!(paths.contains(&"0x00000000000000000000000000000001::Account::Account"));
    }
}
