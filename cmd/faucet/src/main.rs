// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use clap::{value_parser, Parser};
use futures::executor;
use starcoin_faucet::{faucet::Faucet, web};
use starcoin_rpc_client::RpcClient;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_value::TokenValue;
use starcoin_types::account_config::STCUnit;
use std::path::PathBuf;
use tiny_http::Server;

#[derive(Debug, Clone, Parser)]
#[clap(name = "starcoin-faucet", about = "Starcoin")]
pub struct FaucetOpt {
    #[clap(long, short = 'i', value_parser = value_parser!(std::ffi::OsString))]
    pub ipc_path: PathBuf,
    #[clap(long, short = 'a', default_value = "0.0.0.0:8000")]
    pub server_addr: String,
    #[clap(long, short = 'd')]
    pub faucet_address: Option<AccountAddress>,
    #[clap(long, short = 'p', default_value = "")]
    pub faucet_account_password: String,
    #[clap(long, short = 'm', default_value = "1 STC")]
    pub max_amount_pre_request: TokenValue<STCUnit>,
}

fn main() -> Result<()> {
    let _logger_handle = starcoin_logger::init();
    let opts: FaucetOpt = FaucetOpt::parse();
    let client = RpcClient::connect_ipc(opts.ipc_path.as_path()).expect("Failed to connect ipc");

    let account = match opts.faucet_address.as_ref() {
        Some(account_address) => client.account_get(*account_address)?,
        None => client.account_default()?,
    };
    let server = Server::http(&opts.server_addr)
        .unwrap_or_else(|_| panic!("Failed to serve on {}", &opts.server_addr));

    let account = account
        .ok_or_else(|| format_err!("Can not find default account, Please input from account."))?;
    let faucet_address = account.address;
    let faucet = Faucet::new(
        client,
        account,
        opts.faucet_account_password.clone(),
        opts.max_amount_pre_request,
    );
    let fut = web::run(server, faucet);
    println!(
        "Faucet serve on: {}, with faucet account: {}",
        opts.server_addr, faucet_address
    );
    executor::block_on(fut);
    Ok(())
}
