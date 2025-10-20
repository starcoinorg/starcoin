// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use clap::Parser;
use starcoin_account_api::AccountInfo;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RpcClient;
use starcoin_tx_factory::mocker::TxnMocker;
use starcoin_tx_factory::pressure_vm1::start_vm1_pressure_test;
use starcoin_tx_factory::pressure_vm2::start_vm2_pressure_test;
use starcoin_tx_factory::txn_generator::MockTxnGenerator;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::association_address;
use starcoin_vm2_account_api::AccountInfo as AccountInfo2;
use starcoin_vm2_vm_types::account_address::AccountAddress as AccountAddress2;
use starcoin_vm2_vm_types::account_config::association_address as association_address2;
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

    #[clap(
        long,
        help = "account used to send txn, use default account if not specified"
    )]
    pub account_address2: Option<AccountAddress2>,

    #[clap(long, short = 'p', default_value = "")]
    pub account_password: String,

    #[clap(long, default_value = "")]
    pub account_password2: String,

    #[clap(
        long,
        short = 'r',
        help = "address to receive balance, default faucet address"
    )]
    pub receiver_address: Option<AccountAddress>,

    #[clap(long, help = "address to receive balance, default faucet address")]
    pub receiver_address2: Option<AccountAddress2>,

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

    #[clap(long, short = 'v', default_value = "1", help = "vm1 or vm2 test")]
    pub vm_version: u32,
}

fn get_account_or_default2(
    client: &RpcClient,
    account_address: Option<AccountAddress2>,
) -> Result<AccountInfo2> {
    let account = match account_address {
        None => {
            let mut default_account = client.account_default2()?;
            while default_account.is_none() {
                std::thread::sleep(Duration::from_millis(1000));
                default_account = client.account_default2()?;
            }

            // let addr = default_account.clone().unwrap().address;
            // let state_reader = client.state_reader2(StateRootOption::Latest)?;
            // let mut balance = state_reader.get_balance2(addr)?;
            // // balance resource has not been created
            // while balance.is_none() {
            //     std::thread::sleep(Duration::from_millis(1000));
            //     balance = state_reader.get_balance2(addr)?;
            //     info!("account balance is null.");
            // }
            default_account.unwrap()
        }
        Some(a) => match client.account_get2(a)? {
            None => bail!("the specified account does not exists in the starcoin node"),
            Some(w) => w,
        },
    };
    info!("get_account_or_default for vm2: {}", account.address);
    Ok(account)
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

            // let addr = default_account.clone().unwrap().address;
            // let state_reader = client.state_reader(StateRootOption::Latest)?;
            // let mut balance = state_reader.get_balance(addr)?;
            // balance resource has not been created
            // while balance.is_none() {
            //     std::thread::sleep(Duration::from_millis(1000));
            //     balance = state_reader.get_balance(addr)?;
            //     info!("account balance is null.");
            // }
            default_account.unwrap()
        }
        Some(a) => match client.account_get(a)? {
            None => bail!("the specified account does not exists in the starcoin node"),
            Some(w) => w,
        },
    };
    info!("get_account_or_default for vm1: {}", account.address);
    Ok(account)
}

fn main() {
    let _logger_handler = starcoin_logger::init();
    let opts: TxFactoryOpt = TxFactoryOpt::parse();

    let account_address = opts.account_address;
    let account_address2 = opts.account_address2;
    let interval = Duration::from_millis(opts.interval);
    let transfer_account_size = opts.transfer_account_size;
    let account_password = opts.account_password.clone();
    let account_password2 = opts.account_password2.clone();

    let is_stress = opts.stress;
    let mut account_num = opts.account_num;
    let round_num = opts.round_num;
    let vm_version = opts.vm_version;
    if vm_version != 1 && vm_version != 2 {
        panic!("vm version must be 1 or 2");
    }

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
    let account2 = get_account_or_default2(&client, account_address2).unwrap();

    let receiver_address = opts.receiver_address.unwrap_or_else(association_address);
    let receiver_address2 = opts.receiver_address2.unwrap_or_else(association_address2);

    let net = client.node_info().unwrap().net;
    let txn_generator = MockTxnGenerator::new(
        net.chain_id(),
        account.clone(),
        receiver_address,
        account2.clone(),
        receiver_address2,
    );
    let tx_mocker = TxnMocker::new(
        client,
        txn_generator,
        account.address,
        account_password,
        account2.address,
        account_password2,
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

    let handle = if vm_version == 1 {
        start_vm1_pressure_test(
            tx_mocker,
            round_num,
            account_num,
            batch_size,
            interval,
            transfer_account_size,
            is_stress,
            stopping_signal,
        )
    } else if vm_version == 2 {
        start_vm2_pressure_test(
            tx_mocker,
            round_num,
            account_num,
            batch_size,
            interval,
            transfer_account_size,
            is_stress,
            stopping_signal,
        )
    } else {
        panic!("vm version must be 1 or 2");
    };

    handle.join().unwrap();
    info!("txfactory: stop now");
}
