// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use log::debug;
use starcoin_statedb::ChainStateDB;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet, StateSet};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::{
    account_config::AccountResource, language_storage::StructTag, state_view::StateReaderExt,
};

const BLOCK_METADATA_PATH: &str = "0x00000000000000000000000000000001::Block::BlockMetadata";
const CHAIN_ID_PATH: &str = "0x00000000000000000000000000000001::ChainId::ChainId";
const ACCOUNT_RESOURCE_PATH: &str = "0x00000000000000000000000000000001::Account::Account";
const BLOCK_REWARD_QUEUE: &str = "0x00000000000000000000000000000001::BlockReward::RewardQueue";

/// Filter ChainStateSet by removing filtered resources and modifying specific resources
pub fn filter_chain_state_set(
    chain_state_set: ChainStateSet,
    statedb: &ChainStateDB,
) -> anyhow::Result<ChainStateSet> {
    debug!("filtered_state_sets | Entered");
    let mut filtered_state_set_vec = Vec::new();
    for (address, account_state_set) in chain_state_set.state_sets() {
        let mut filtered_resource_state_set = vec![];

        if let Some(resource_set) = account_state_set.resource_set() {
            for (key, blob) in resource_set.iter() {
                let struct_tag = bcs_ext::from_bytes::<StructTag>(key)?;
                let struct_tag_str = format!(
                    "{}::{}::{}",
                    struct_tag.address, struct_tag.module, struct_tag.name
                );

                let filtered_blob = if ACCOUNT_RESOURCE_PATH == struct_tag_str {
                    bcs_ext::to_bytes(
                        &bcs_ext::from_bytes::<AccountResource>(blob)?.clone_with_zero_seq_number(),
                    )?
                } else if BLOCK_METADATA_PATH == struct_tag_str {
                    bcs_ext::to_bytes(&statedb.get_block_metadata()?)?
                } else if BLOCK_REWARD_QUEUE == struct_tag_str {
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
                } else if CHAIN_ID_PATH == struct_tag_str {
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
