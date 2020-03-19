// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod access_path;
pub mod account_address;
pub mod account_config;
pub mod account_state;
pub mod block;
pub mod block_metadata;
pub mod byte_array;
pub mod change_set;
pub mod contract_event;
pub mod event;
pub mod ids;
pub mod language_storage;
pub mod peer_info;
pub mod proof;
pub mod startup_info;
pub mod state_set;
pub mod system_events;
pub mod transaction;
pub mod vm_error;
pub mod write_set;
pub use ethereum_types::{H256, U256};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
