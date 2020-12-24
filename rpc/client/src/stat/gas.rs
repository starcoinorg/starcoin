// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("args error");
        return;
    }
    let ipc_file = &args[1];
    let client = Arc::new(RpcClient::connect_ipc(ipc_file).expect("Connect by ipc fail."));
    let chain_info = client.chain_info().unwrap();
    let end_number = chain_info.head.number;
    // get tps
    let mut block_number = 1;
    while block_number < end_number {
        let block = client
            .clone()
            .chain_get_block_by_number(block_number)
            .unwrap();
        println!("{:?}, {:?}", block.header.number, block.header.gas_used);
        block_number += 1;
    }
}
