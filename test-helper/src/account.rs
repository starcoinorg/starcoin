// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use network_p2p_core::export::log::debug;
use starcoin_types::{
    account_address::AccountAddress,
    language_storage::StructTag,
    state_set::{AccountStateSet, ChainStateSet, StateSet},
};

pub fn print_chain_state_set(
    chain_state_set: &ChainStateSet,
    match_address: Option<AccountAddress>,
) -> anyhow::Result<()> {
    debug!(
        "print_chain_state_set | Entered, chain_state_set accounts: {:?}",
        chain_state_set.len()
    );

    for (account_address, account_state_set) in chain_state_set.state_sets() {
        if let Some(match_address) = match_address {
            if match_address != *account_address {
                continue;
            }
        }
        print_account_state_set(account_address, account_state_set)?;
    }
    Ok(())
}

pub fn print_account_state_set(
    address: &AccountAddress,
    account_state_set: &AccountStateSet,
) -> anyhow::Result<()> {
    debug!("print_account_state_set | Entered: {:?}", address);

    if let Some(code_set) = account_state_set.code_set() {
        print_code_state_set(code_set)?;
    } else {
        debug!("no code found in account_state_set");
    }

    if let Some(resource_state) = account_state_set.resource_set() {
        print_resource_state_set(resource_state)?;
    } else {
        debug!("no resource found in account_state_set");
    }

    debug!("print_account_state_set | Exited: {:?}", address);

    Ok(())
}

pub fn print_resource_state_set(state_set: &StateSet) -> anyhow::Result<()> {
    debug!("print_resource_state_set | count: {:?}", state_set.len());
    Ok(())
}

pub fn print_code_state_set(state_set: &StateSet) -> anyhow::Result<()> {
    debug!("print_code_state_set | count: {:?}", state_set.len());
    Ok(())
}
//
// /// Helper function to print account resource set
// pub fn print_account_resource_set(
//     statedb: &ChainStateDB,
//     address: &AccountAddress,
// ) -> anyhow::Result<()> {
//     debug!("=== Printing resource set for account {} ===", address);
//
//     let account_state_set = statedb.get_account_state_set(address)?;
//     match account_state_set {
//         None => {
//             debug!("Account {} does not exist", address);
//             return Ok(());
//         }
//         Some(state_set) => {
//             if let Some(resource_set) = state_set.resource_set() {
//                 debug!(
//                     "Found {} resources for account {}",
//                     resource_set.len(),
//                     address
//                 );
//
//                 for (key, value) in resource_set.iter() {
//                     // Decode the struct tag from the key
//                     let struct_tag = bcs_ext::from_bytes::<StructTag>(key.as_slice())?;
//                     debug!("Resource type: {}", struct_tag);
//
//                     // Try to decode and print the resource value
//                     match playground::view_resource(statedb, struct_tag.clone(), value.as_slice()) {
//                         Ok(annotated_struct) => {
//                             debug!("Resource value: {:#?}", annotated_struct);
//                         }
//                         Err(e) => {
//                             debug!("Failed to decode resource {}: {:?}", struct_tag, e);
//                             debug!("Raw value (hex): {}", hex::encode(value));
//                         }
//                     }
//                     debug!("---");
//                 }
//             } else {
//                 debug!("No resource set found for account {}", address);
//             }
//         }
//     }
//
//     debug!("=== End of resource set for account {} ===", address);
//     Ok(())
// }

/// Utility function to print BCS decoded resource information
/// This function mimics the process in migrate_legacy_state_data but focuses on printing
/// the decoded resource information instead of applying it to statedb
pub fn print_bcs_decoded_resources(bcs_content: Vec<u8>) -> anyhow::Result<()> {
    // Decode ChainStateSet from BCS (same as migrate_legacy_state_data)
    let chain_state_set: ChainStateSet = bcs_ext::from_bytes(&bcs_content)?;
    debug!(
        "Successfully decoded ChainStateSet with {} account states",
        chain_state_set.len()
    );

    // Print detailed resource information for each account
    for (account_address, account_state_set) in chain_state_set.state_sets() {
        debug!("=== Account: {} ===", account_address);

        let resource_set = account_state_set.resource_set();
        if resource_set.is_none() {
            continue;
        }
        let resource_set = account_state_set.resource_set().unwrap();
        debug!("  Found {} resources", resource_set.len());

        for (key, value) in resource_set.iter() {
            // Decode the struct tag from the key
            match bcs_ext::from_bytes::<StructTag>(key.as_slice()) {
                Ok(struct_tag) => {
                    debug!(
                        "  Resource type: {}, size: {} bytes, Raw value (hex): {}",
                        struct_tag,
                        value.len(),
                        hex::encode(value)
                    );

                    // Try to decode as some common resource types
                    if let Ok(version) = bcs_ext::from_bytes::<
                        starcoin_vm_types::on_chain_config::Version,
                    >(value.as_slice())
                    {
                        debug!("  Decoded as Version: {:?}", version);
                    } else if let Ok(balance) = bcs_ext::from_bytes::<
                        starcoin_vm_types::account_config::BalanceResource,
                    >(value.as_slice())
                    {
                        debug!("  Decoded as BalanceResource: {:?}", balance);
                    } else if let Ok(account) = bcs_ext::from_bytes::<
                        starcoin_vm_types::account_config::AccountResource,
                    >(value.as_slice())
                    {
                        debug!("  Decoded as AccountResource: {:?}", account);
                    } else if let Ok(token_info) = bcs_ext::from_bytes::<
                        starcoin_vm_types::account_config::TokenInfo,
                    >(value.as_slice())
                    {
                        debug!("  Decoded as TokenInfo: {:?}", token_info);
                    } else {
                        debug!("  Could not decode as common resource types");
                    }
                }
                Err(e) => {
                    debug!(
                        "Failed to decode struct tag: Raw key (hex): {}, error: {:?}, ",
                        hex::encode(key),
                        e
                    );
                }
            }
        }
        debug!("=== End Account: {} ===", account_address);
    }
    Ok(())
}
