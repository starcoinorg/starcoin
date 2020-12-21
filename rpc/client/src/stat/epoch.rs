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
    let mut epoch_number = 1;
    while epoch_number < epoch_count {
        let mut block_number = epoch_number * 240 - 1;
        if block_number >= end_number {
            block_number = end_number;
        }
        let epoch = client
            .clone()
            .get_epoch_info_by_number(block_number)
            .unwrap();
        println!(
            "epoch: {:?}, {:?}, {:?}",
            epoch.number(),
            epoch.block_time_target(),
            epoch.epoch_data().uncles()
        );
        epoch_number += 1;
    }
}
