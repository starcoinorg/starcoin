use ctrlc;
use starcoin_executor::executor::Executor;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RemoteStateReader;
use starcoin_rpc_client::RpcClient;
use starcoin_state_api::AccountStateReader;
use starcoin_tx_factory::txn_generator::MockTxnGenerator;
use starcoin_types::account_address::AccountAddress;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Default)]
#[structopt(name = "txfactory", about = "tx generator for starcoin")]
pub struct TxFactoryOpt {
    #[structopt(long, short = "p", parse(from_os_str))]
    pub ipc_path: PathBuf,
    // #[structopt(long, short = "a", default_value = "0.0.0.0:8000")]
    // pub server_addr: String,
    #[structopt(
        long,
        short = "i",
        default_value = "1000",
        help = "interval(in ms) of txn gen"
    )]
    pub interval: u64,
    #[structopt(long, short = "d", default_value = "0xa550c18")]
    pub faucet_address: String,
}

fn main() {
    let _logger_handler = starcoin_logger::init();
    let opts: TxFactoryOpt = TxFactoryOpt::from_args();
    let faucet_address = AccountAddress::from_hex_literal(&opts.faucet_address)
        .expect("a hex encoded address start with 0x");
    let interval = Duration::from_millis(opts.interval);
    let client = RpcClient::connect_ipc(opts.ipc_path).expect("ipc connect success");
    let stopping_signal = Arc::new(AtomicBool::new(false));
    let stopping_signal_clone = stopping_signal.clone();
    ctrlc::set_handler(move || {
        stopping_signal_clone.store(true, Ordering::SeqCst);
    })
    .unwrap();
    let handle = std::thread::spawn(move || {
        let state_reader = RemoteStateReader::new(&client);
        let account_state_reader = AccountStateReader::new(&state_reader);
        let txn_generator = MockTxnGenerator::new(faucet_address);
        while !stopping_signal.load(Ordering::SeqCst) {
            let txn = txn_generator
                .generate_mock_txn::<Executor>(&account_state_reader)
                .expect("generate ok");
            let success = client
                .submit_transaction(txn.as_signed_user_txn().unwrap().clone())
                .unwrap();
            println!("success: {}", success);
            std::thread::sleep(interval);
        }
    });
    handle.join().unwrap();
    info!("txfactory: stop now");
}
