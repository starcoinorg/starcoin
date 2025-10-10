// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use clap::Parser;
use starcoin_account_api::AccountInfo;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RpcClient;
use starcoin_rpc_client::StateRootOption;
use starcoin_state_api::StateReaderExt;
use starcoin_tx_factory::pressure_vm1::TxnMocker;
use starcoin_tx_factory::txn_generator::MockTxnGenerator;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::association_address;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Parser, Default)]
#[clap(name = "txfactory", about = "tx generator for starcoin")]
pub struct TxFactoryOpt {
    #[clap(long)]
    pub ipc_path: PathBuf,
    #[clap(
        long,
        short = 'i',
        default_value = "1000",
        help = "interval(in ms) of txn gen"
    )]
    pub interval: u64,
    #[clap(
        long,
        short = 'f',
        default_value = "800",
        help = "The number of accounts participating in each round of transactions: the first half are sender accounts, and the second half are receiver accounts, with a default total of 800."
    )]
    pub transfer_account_size: usize,
    #[clap(
        long,
        short = 'a',
        help = "account used to send txn, use default account if not specified"
    )]
    pub account_address: Option<AccountAddress>,
    #[clap(long, short = 'p', default_value = "")]
    pub account_password: String,

    #[clap(
        long,
        short = 'r',
        help = "address to receive balance, default faucet address"
    )]
    pub receiver_address: Option<AccountAddress>,

    #[clap(long, short = 'k', help = "this option is deprecated")]
    pub _receiver_public_key: Option<String>,

    #[clap(long = "stress", short = 's', help = "is stress test or not")]
    pub stress: bool,

    #[clap(
        long,
        short = 'n',
        default_value = "30",
        help = "numbers of account will be created"
    )]
    pub account_num: u32,

    #[clap(
        long,
        short = 't',
        default_value = "20",
        help = "count of round number"
    )]
    pub round_num: u32,
    #[clap(long, short = 'w', default_value = "60", help = "watch_timeout")]
    pub watch_timeout: u32,
    #[clap(
        long,
        short = 'b',
        default_value = "50",
        help = "create account batch size"
    )]
    pub batch_size: u32,
}

fn get_account_or_default(
    client: &RpcClient,
    account_address: Option<AccountAddress>,
) -> Result<AccountInfo> {
    let account = match account_address {
        None => {
            let mut default_account = client.account_default()?;
            while default_account.is_none() {
                std::thread::sleep(Duration::from_millis(1000));
                default_account = client.account_default()?;
            }

            let addr = default_account.clone().unwrap().address;
            let state_reader = client.state_reader(StateRootOption::Latest)?;
            let mut balance = state_reader.get_balance(addr)?;
            // balance resource has not been created
            while balance.is_none() {
                std::thread::sleep(Duration::from_millis(1000));
                balance = state_reader.get_balance(addr)?;
                info!("account balance is null.");
            }
            default_account.unwrap()
        }
        Some(a) => match client.account_get(a)? {
            None => bail!("the specified account does not exists in the starcoin node"),
            Some(w) => w,
        },
    };
    info!("get_account_or_default: {}", account.address);
    Ok(account)
}

fn main() {
    let _logger_handler = starcoin_logger::init();
    let opts: TxFactoryOpt = TxFactoryOpt::parse();

    let account_address = opts.account_address;
    let interval = Duration::from_millis(opts.interval);
    let transfer_account_size = opts.transfer_account_size;
    let account_password = opts.account_password.clone();

    let is_stress = opts.stress;
    let mut account_num = opts.account_num;
    let round_num = opts.round_num;

    if !is_stress {
        account_num = 0;
    }
    let watch_timeout = opts.watch_timeout;
    let batch_size = opts.batch_size;

    let mut connected = RpcClient::connect_ipc(opts.ipc_path.clone());
    while connected.is_err() {
        std::thread::sleep(Duration::from_millis(1000));
        connected = RpcClient::connect_ipc(opts.ipc_path.clone());
        info!("re connecting...");
    }
    let client = connected.unwrap();

    let account = get_account_or_default(&client, account_address).unwrap();

    let receiver_address = opts.receiver_address.unwrap_or_else(association_address);

    let net = client.node_info().unwrap().net;
    let txn_generator = MockTxnGenerator::new(net.chain_id(), account.clone(), receiver_address);
    let tx_mocker = TxnMocker::new(
        client,
        txn_generator,
        account.address,
        account_password,
        Duration::from_secs(86400), // 24 hours
        watch_timeout,
    );

    let tx_mocker = match tx_mocker {
        Ok(t) => t,
        Err(e) => {
            panic!("mocker init error: {:?}", e);
        }
    };

    let stopping_signal = Arc::new(AtomicBool::new(false));
    let stopping_signal_clone = stopping_signal.clone();
    ctrlc::set_handler(move || {
        stopping_signal_clone.store(true, Ordering::SeqCst);
    })
    .unwrap();

    let handle = start_vm1_pressure_test(
        tx_mocker,
        round_num,
        account_num,
        batch_size,
        interval,
        transfer_account_size,
        is_stress,
        stopping_signal,
    );

    handle.join().unwrap();
    info!("txfactory: stop now");
}

fn start_vm1_pressure_test(
    mut tx_mocker: TxnMocker,
    round_num: u32,
    account_num: u32,
    batch_size: u32,
    interval: Duration,
    transfer_account_size: usize,
    is_stress: bool,
    stopping_signal: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let accounts = tx_mocker
            .get_or_create_accounts(account_num, batch_size)
            .expect("create accounts should success");
        while !stopping_signal.load(Ordering::SeqCst) {
            if tx_mocker.get_factory_status() {
                if is_stress {
                    info!("stress account: {}", accounts.len());
                    let success = tx_mocker.stress_test(
                        accounts.clone(),
                        round_num,
                        interval,
                        transfer_account_size,
                    );
                    if let Err(e) = success {
                        error!("fail to run stress test, err: {:?}", &e);
                        // if txn is rejected, recheck sequence number, and start over
                        if let Err(e) = tx_mocker.recheck_sequence_number() {
                            error!("fail to start over, err: {:?}", e);
                        }
                    }
                } else {
                    let success = tx_mocker.gen_and_submit_txn(false);
                    if let Err(e) = success {
                        error!("fail to generate/submit mock txn, err: {:?}", &e);
                        // if txn is rejected, recheck sequence number, and start over
                        if let Err(e) = tx_mocker.recheck_sequence_number() {
                            error!("fail to start over, err: {:?}", e);
                        }
                    }
                }
            } else {
                info!("txfactory is stop.");
            }

            std::thread::sleep(interval);
        }
    })
}
