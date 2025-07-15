// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use log::debug;
use starcoin_statedb::ChainStateDB;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet, StateSet};
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::{
    access_path::AccessPath, account_config::AccountResource, identifier::Identifier,
    language_storage::StructTag, state_store::state_key::StateKey, state_view::StateReaderExt,
    state_view::StateView,
};

/// Filter ChainStateSet by removing filtered resources and modifying specific resources
pub fn filter_chain_state_set(
    chain_state_set: ChainStateSet,
    statedb: &ChainStateDB,
) -> anyhow::Result<ChainStateSet> {
    let mut filtered_state_set_vec = Vec::new();
    for (address, account_state_set) in chain_state_set.state_sets() {
        let mut filtered_resource_state_set = vec![];

        if let Some(resource_set) = account_state_set.resource_set() {
            for (key, blob) in resource_set.iter() {
                let struct_tag = bcs_ext::from_bytes::<StructTag>(key)?;

                let filtered_blob = if struct_tag.address == genesis_address()
                    && struct_tag.module == Identifier::new("Account")?
                    && struct_tag.name == Identifier::new("Account")?
                {
                    bcs_ext::to_bytes(
                        &bcs_ext::from_bytes::<AccountResource>(blob)?.clone_with_zero_seq_number(),
                    )?
                } else if struct_tag.address == genesis_address()
                    && struct_tag.module == Identifier::new("Block")?
                    && struct_tag.name == Identifier::new("BlockMetadata")?
                {
                    bcs_ext::to_bytes(&statedb.get_block_metadata()?)?
                } else if struct_tag.address == genesis_address()
                    && struct_tag.module == Identifier::new("BlockReward")?
                    && struct_tag.name == Identifier::new("RewardQueue")?
                {
                    // Get local block reward queue data from current state
                    let state_key = StateKey::AccessPath(AccessPath::new(
                        *address,
                        starcoin_types::access_path::DataPath::Resource(struct_tag),
                    ));
                    match statedb.get_state_value(&state_key)? {
                        Some(local_data) => local_data.to_vec(),
                        None => {
                            debug!("No local block reward queue data found, using empty data");
                            blob.clone()
                        }
                    }
                } else if struct_tag.address == genesis_address()
                    && struct_tag.module == Identifier::new("ChainId").unwrap()
                    && struct_tag.name == Identifier::new("ChainId").unwrap()
                {
                    bcs_ext::to_bytes(&statedb.get_chain_id()?)?
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
    debug!("filtered_state_sets | Exited");
    Ok(ChainStateSet::new(filtered_state_set_vec))
}
