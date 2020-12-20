// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;
use tokio_compat::runtime::Runtime;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("args error");
        return;
    }
    let mut rt = Runtime::new().unwrap();
    let ipc_file = &args[1];
    let client = Arc::new(RpcClient::connect_ipc(ipc_file, &mut rt).expect("Connect by ipc fail."));
    let chain_info = client.chain_info().unwrap();
    let end_number = chain_info.head.number;
    let epoch_count = end_number / 240 + 1;
    // get tps
    let mut epoch = 1;
    while epoch < epoch_count {
        let mut block_number = epoch * 240 - 1;
        if block_number >= end_number {
            block_number = end_number;
        }
        let tps = client.clone().tps(Some(block_number)).unwrap();
        println!("tps: {:?}", tps);
        epoch += 1;
    }
}
