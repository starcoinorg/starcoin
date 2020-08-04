use futures::executor;
use starcoin_faucet::{faucet::Faucet, web};
use starcoin_rpc_client::RpcClient;
use starcoin_types::account_address::AccountAddress;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use tiny_http::Server;

#[derive(Debug, Clone, StructOpt, Default)]
#[structopt(name = "starcoin", about = "Starcoin")]
pub struct FaucetOpt {
    #[structopt(long, short = "p", parse(from_os_str))]
    pub ipc_path: PathBuf,
    #[structopt(long, short = "a", default_value = "0.0.0.0:8000")]
    pub server_addr: String,
    #[structopt(long, short = "d")]
    pub faucet_address: String,
}

fn main() {
    let _logger_handle = starcoin_logger::init();
    let opts: FaucetOpt = FaucetOpt::from_args();
    let mut runtime = tokio_compat::runtime::Runtime::new().unwrap();
    let account_address =
        AccountAddress::from_str(&opts.faucet_address).expect("Invalid faucet address");
    let server = Server::http(&opts.server_addr)
        .unwrap_or_else(|_| panic!("Failed to serve on {}", opts.server_addr));
    let client =
        RpcClient::connect_ipc(opts.ipc_path, &mut runtime).expect("Failed to connect ipc");
    let account = client
        .account_get(account_address)
        .unwrap()
        .expect("Invalid faucet account address");
    let faucet = Faucet::new(client, account);
    let fut = web::run(server, faucet);
    println!(
        "Faucet serve on: {}, with faucet account: {}",
        opts.server_addr, opts.faucet_address
    );
    executor::block_on(fut);
}
