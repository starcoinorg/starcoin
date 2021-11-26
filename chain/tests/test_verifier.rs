// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_chain_mock::MockChain;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_rpc_api::types::BlockView;
use starcoin_types::block::Block;
use std::convert::TryInto;

#[test]
fn test_main_static_block_verify() {
    let mut mock_chain = MockChain::new(ChainNetwork::new_builtin(BuiltinNetworkID::Main)).unwrap();
    let blocks = vec![
        serde_json::json!({"header": {"block_hash": "0xc2ce2515f48a2ffef2e69b0288bb2aa78c8bfa9a55312eb948758702cd55f805","parent_hash": "0x80848150abee7e9a3bfe9542a019eb0b8b01f124b63b011f9c338fdb935c417d","timestamp": "1621311490074","number": "1","author": "0x34e2fd022578411aa8c249f8dc1da714","author_auth_key": "0x250fbfa3f8e7c143f0658208cee3529d34e2fd022578411aa8c249f8dc1da714","txn_accumulator_root": "0x52bf3ebdb38f43690d8468ace3b5863b19cd17a5a4a50a0af30d0c1cb47623e9","block_accumulator_root": "0x80848150abee7e9a3bfe9542a019eb0b8b01f124b63b011f9c338fdb935c417d","state_root": "0x7244a297682da309e05bdd30a71876414cab8d499f5a904817bcd823307ad560","gas_used": "0","difficulty": "0xb1ec37","body_hash": "0xc01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97","chain_id": 1,"nonce": 1470858403,"extra": "0x00000000"},"body": {"Full": []},"uncles": []}),
        serde_json::json!(
                  {
          "header": {
            "block_hash": "0xdda5d70eac37fa646fd9cebb274acf98fba78ff1e8afeb8b4ac11a1d90ffc84c",
            "parent_hash": "0xc2ce2515f48a2ffef2e69b0288bb2aa78c8bfa9a55312eb948758702cd55f805",
            "timestamp": "1621311561476",
            "number": "2",
            "author": "0x000dc78e982dcdc5c80246f76d2140aa",
            "author_auth_key": "0x38e115d617f293f23ae8d951330e97a3000dc78e982dcdc5c80246f76d2140aa",
            "txn_accumulator_root": "0x40e0bb9454a40a6a2dd0579a60557c419bec995dfd80a81452a2fe0cd344ec1d",
            "block_accumulator_root": "0xe2d703b76eb066b4453eb6b05f90f9fe498d85afbafc97110cc946702bf10459",
            "state_root": "0xa4be7b3d401da8ed15f7c4fa7c06a7cd25815f246b62fb0e7f05f32a59739c89",
            "gas_used": "0",
            "difficulty": "0xb1ec37",
            "body_hash": "0xc01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97",
            "chain_id": 1,
            "nonce": 3221282425u32,
            "extra": "0x00000000"
          },
          "body": {
            "Full": []
          },
          "uncles": []
        }),
    ];
    for block_json in blocks {
        let block_view = serde_json::from_value::<BlockView>(block_json).unwrap();
        let block: Block = block_view.try_into().unwrap();
        let result = mock_chain.apply(block.clone());
        assert!(
            result.is_ok(),
            "verify block: {:?} error: {:?}",
            block,
            result.err().unwrap()
        );
    }
}
