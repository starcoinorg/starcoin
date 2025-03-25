// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use starcoin_types::access_path::AccessPath;
use std::str::FromStr;

pub use chain_state::{
    AccountStateReader, ChainStateReader, ChainStateWriter, StateProof, StateWithProof,
    StateWithTableItemProof,
};
pub use starcoin_state_tree::StateNodeStore;
use starcoin_vm_types::access_path::DataPath;
use starcoin_vm_types::account_config::TABLE_HANDLE_ADDRESS_LIST;
pub use starcoin_vm_types::state_view::StateReaderExt;

mod chain_state;

pub static TABLE_PATH_LIST: Lazy<Vec<DataPath>> = Lazy::new(|| {
    let mut path_list = vec![];
    for handle_address in &*TABLE_HANDLE_ADDRESS_LIST {
        let str = format!(
            "{}/1/{}::TableHandles::TableHandles",
            handle_address, handle_address,
        );
        path_list.push(AccessPath::from_str(str.as_str()).unwrap().path);
    }
    path_list
});

#[cfg(test)]
mod tests {
    use crate::TABLE_PATH_LIST;

    #[test]
    fn test_table_path_list() {
        let mut path_list = vec![];
        let str_list = vec![
            "1/0x00000000000000000000000000000031::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000032::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000033::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000034::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000035::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000036::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000037::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000038::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000039::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000003a::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000003b::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000003c::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000003d::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000003e::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000003f::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000040::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000041::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000042::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000043::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000044::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000045::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000046::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000047::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000048::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000049::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000004a::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000004b::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000004c::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000004d::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000004e::TableHandles::TableHandles",
            "1/0x0000000000000000000000000000004f::TableHandles::TableHandles",
            "1/0x00000000000000000000000000000050::TableHandles::TableHandles",
        ];
        for table_path in TABLE_PATH_LIST.iter() {
            path_list.push(format!("{}", table_path));
        }
        assert_eq!(path_list, str_list);
    }
}
