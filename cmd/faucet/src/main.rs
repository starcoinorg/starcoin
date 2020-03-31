use tiny_http::Server;
use futures::executor;
use starcoin_faucet::{faucet::Faucet, web};
use starcoin_rpc_client::RpcClient;
use starcoin_types::account_address::AccountAddress;
use starcoin_logger::prelude::*;
use std::str::FromStr;

fn main() {
    let _logger_handle = starcoin_logger::init();
    let ipc_path = "/Users/fikgol/workspaces/starcoin/starcoin/starcoin.ipc";
    let account_address = AccountAddress::from_str("3587b0ae39192741f91e333d845bfeff").unwrap();
    let server_addr = "0.0.0.0:8000";

    let server = Server::http(server_addr).unwrap();
    let client = RpcClient::connect_ipc(ipc_path).unwrap();
    let account = client.account_get(account_address).unwrap().unwrap();
    let faucet = Faucet::new(client, account);
    let fut = web::run(server, faucet);
    executor::block_on(fut);
}