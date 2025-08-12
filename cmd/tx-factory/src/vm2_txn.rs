// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Context};
use starcoin_config::Connect;
use starcoin_rpc_client::{AsyncRpcClient, ConnSource};
use starcoin_tx_factory::vm2_txn_lib::{async_main, generate_cmd, FUNDING_ACCOUNT};
use starcoin_vm2_types::account_address::AccountAddress;
use std::str::FromStr;
use std::sync::Arc;

async fn create_client(node_url: &str) -> anyhow::Result<AsyncRpcClient> {
    let connect = Connect::from_str(node_url)?;
    match connect {
        Connect::IPC(Some(ipc_path)) => AsyncRpcClient::new(ConnSource::Ipc(ipc_path)).await,
        Connect::WebSocket(url) => AsyncRpcClient::new(ConnSource::WebSocket(url)).await,
        _ => Err(anyhow!("Unsupported connection type: {:?}", connect)),
    }
}

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    starcoin_logger::init();
    let mut args = std::env::args().skip(1);
    let sub_cmd = args.next().context("sub command")?;
    let handle = match sub_cmd.as_str() {
        "generate" => {
            let account_path = args.next().context("account file path")?;
            let count = args
                .next()
                .context("count of transactions to generate")?
                .parse::<usize>()
                .context("invalid count")?;

            rt.spawn(generate_cmd(account_path, count))
        }
        "run" => {
            let node_url = args.next().context("node url")?;
            let account_path = args.next().context("account file path")?;
            let target_address = args
                .next()
                .map(|s| s.parse::<AccountAddress>())
                .transpose()?
                .unwrap_or(FUNDING_ACCOUNT.address());
            let client = rt.block_on(create_client(&node_url))?;

            rt.spawn(async_main(Arc::new(client), target_address, account_path))
        }
        _ => return Err(anyhow!("Unknown command: {}", sub_cmd)),
    };

    if let Err(e) = rt.block_on(handle) {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }

    Ok(())
}
