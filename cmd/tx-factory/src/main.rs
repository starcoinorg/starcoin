// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use starcoin_account_api::AccountInfo;
use starcoin_crypto::HashValue;
use starcoin_executor::DEFAULT_EXPIRATION_TIME;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::types::FactoryAction;
use starcoin_rpc_client::RpcClient;
use starcoin_rpc_client::StateRootOption;
use starcoin_state_api::{ChainStateReader, StateReaderExt};
use starcoin_tx_factory::txn_generator::MockTxnGenerator;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::association_address;
use starcoin_types::transaction::RawUserTransaction;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Default)]
#[structopt(name = "txfactory", about = "tx generator for starcoin")]
pub struct TxFactoryOpt {
    #[structopt(long, parse(from_os_str))]
    pub ipc_path: PathBuf,
    #[structopt(
        long,
        short = "i",
        default_value = "1000",
        help = "interval(in ms) of txn gen"
    )]
    pub interval: u64,
    #[structopt(
        long,
        short = "a",
        help = "account used to send txn, use default account if not specified"
    )]
    pub account_address: Option<AccountAddress>,
    #[structopt(long, short = "p", default_value = "")]
    pub account_password: String,

    #[structopt(
        long,
        short = "r",
        help = "address to receive balance, default faucet address"
    )]
    pub receiver_address: Option<AccountAddress>,

    #[structopt(long, short = "k", help = "this option is deprecated")]
    pub _receiver_public_key: Option<String>,

    #[structopt(long = "stress", short = "s", help = "is stress test or not")]
    pub stress: bool,

    #[structopt(
        long,
        short = "n",
        default_value = "30",
        help = "numbers of account will be created"
    )]
    pub account_num: u32,

    #[structopt(
        long,
        short = "t",
        default_value = "20",
        help = "count of round number"
    )]
    pub round_num: u32,
    #[structopt(long, short = "w", default_value = "60", help = "watch_timeout")]
    pub watch_timeout: u32,
    #[structopt(
        long,
        short = "b",
        default_value = "50",
        help = "create account batch size"
    )]
    pub batch_size: u32,
}

const INITIAL_BALANCE: u128 = 1_000_000_000;

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
    let opts: TxFactoryOpt = TxFactoryOpt::from_args();

    let account_address = opts.account_address;
    let interval = Duration::from_millis(opts.interval);
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
    while matches!(connected, Err(_)) {
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
        Duration::from_secs(60 * 10),
        watch_timeout,
    );

    let mut tx_mocker = match tx_mocker {
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
    let handle = std::thread::spawn(move || {
        let accounts = tx_mocker
            .get_or_create_accounts(account_num, batch_size)
            .expect("create accounts should success");
        while !stopping_signal.load(Ordering::SeqCst) {
            if tx_mocker.get_factory_status() {
                if is_stress {
                    info!("stress account: {}", accounts.len());
                    let success = tx_mocker.stress_test(accounts.clone(), round_num);
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
    });
    handle.join().unwrap();
    info!("txfactory: stop now");
}

struct TxnMocker {
    client: RpcClient,
    generator: MockTxnGenerator,
    account_address: AccountAddress,
    account_password: String,
    unlock_duration: Duration,

    next_sequence_number: u64,
    account_unlock_time: Option<Instant>,
    watch_timeout: u32,
}

impl TxnMocker {
    pub fn new(
        client: RpcClient,
        generator: MockTxnGenerator,
        account_address: AccountAddress,
        account_password: String,
        unlock_duration: Duration,
        watch_timeout: u32,
    ) -> Result<Self> {
        let state_reader = client.state_reader(StateRootOption::Latest)?;

        let account_resource = state_reader.get_account_resource(account_address)?;
        if account_resource.is_none() {
            bail!("account {} not exists, please faucet it", account_address);
        }
        let account_resource = account_resource.unwrap();
        let mut next_sequence_number = account_resource.sequence_number();
        // if txpool already has some future txn, use the sequence number after that.
        let seq_number_in_txpool = client.next_sequence_number_in_txpool(account_address)?;
        if let Some(n) = seq_number_in_txpool {
            if n > next_sequence_number {
                next_sequence_number = n;
            }
        }
        Ok(Self {
            client,
            generator,
            account_address,
            account_password,
            unlock_duration,
            account_unlock_time: None,
            next_sequence_number,
            watch_timeout,
        })
    }
}

impl TxnMocker {
    fn fetch_expiration_time(&self) -> u64 {
        let node_info = self
            .client
            .node_info()
            .expect("node_info() should not failed");
        node_info.now_seconds + DEFAULT_EXPIRATION_TIME
    }
    fn get_factory_status(&self) -> bool {
        self.client
            .debug_txfactory_status(FactoryAction::Status)
            .unwrap()
    }
    fn recheck_sequence_number(&mut self) -> Result<()> {
        let seq_number_in_pool = self
            .client
            .next_sequence_number_in_txpool(self.account_address)?;

        self.next_sequence_number = match seq_number_in_pool {
            Some(n) => n,
            None => {
                let state_reader = self.client.state_reader(StateRootOption::Latest)?;

                let account_resource = state_reader.get_account_resource(self.account_address)?;
                if account_resource.is_none() {
                    bail!(
                        "account {} not exists, please faucet it",
                        &self.account_address
                    );
                }
                account_resource.unwrap().sequence_number()
            }
        };
        Ok(())
    }

    fn gen_and_submit_txn(&mut self, blocking: bool) -> Result<HashValue> {
        let expiration_timestamp = self.fetch_expiration_time();
        let raw_txn = self
            .generator
            .generate_mock_txn(self.next_sequence_number, expiration_timestamp)?;
        info!("prepare to sign txn, sender: {}", raw_txn.sender());

        self.unlock_account()?;

        let user_txn = match self.client.account_sign_txn(raw_txn) {
            Err(e) => {
                // sign txn fail, we should unlock again
                self.account_unlock_time = None;
                return Err(e);
            }
            Ok(txn) => txn,
        };
        info!(
            "prepare to submit txn, sender:{},seq:{}",
            user_txn.sender(),
            user_txn.sequence_number(),
        );
        let txn_hash = user_txn.id();
        let result = self.client.submit_transaction(user_txn);

        // increase sequence number if added in pool.
        if matches!(result, Ok(_)) {
            self.next_sequence_number += 1;
        }
        if blocking {
            self.client.watch_txn(
                txn_hash,
                Some(Duration::from_secs(self.watch_timeout as u64)),
            )?;
        }
        result
    }

    fn unlock_account(&mut self) -> Result<()> {
        let unlock_time = self.account_unlock_time;
        match unlock_time {
            Some(t) if t + self.unlock_duration > Instant::now() => {}
            _ => {
                // reset first just in case account_unlock fail
                self.account_unlock_time = None;

                let new_unlock_time = Instant::now();
                // try unlock account
                self.client.account_unlock(
                    self.account_address,
                    self.account_password.clone(),
                    self.unlock_duration,
                )?;

                self.account_unlock_time = Some(new_unlock_time);
            }
        }
        Ok(())
    }

    fn submit_txn(
        &self,
        raw_txn: RawUserTransaction,
        sender: AccountAddress,
        blocking: bool,
    ) -> Result<HashValue> {
        // try unlock account
        self.client
            .account_unlock(sender, self.account_password.clone(), self.unlock_duration)?;

        let user_txn = match self.client.account_sign_txn(raw_txn) {
            Err(e) => {
                return Err(e);
            }
            Ok(txn) => txn,
        };
        info!(
            "prepare to submit txn, sender:{},seq:{}",
            user_txn.sender(),
            user_txn.sequence_number(),
        );
        let txn_hash = user_txn.id();
        let result = self.client.submit_transaction(user_txn);

        if matches!(result, Ok(_)) && blocking {
            self.client.watch_txn(
                txn_hash,
                Some(Duration::from_secs(self.watch_timeout as u64)),
            )?;
        }
        result
    }

    fn gen_and_submit_transfer_txn(
        &self,
        sender: AccountAddress,
        receiver_address: AccountAddress,
        amount: u128,
        gas_price: u64,
        sequence_number: u64,
        blocking: bool,
        expiration_timestamp: u64,
    ) -> Result<HashValue> {
        let raw_txn = self.generator.generate_transfer_txn(
            sequence_number,
            sender,
            receiver_address,
            amount,
            gas_price,
            expiration_timestamp,
        )?;
        info!("prepare to sign txn, sender: {}", raw_txn.sender());
        self.submit_txn(raw_txn, sender, blocking)
    }

    fn get_or_create_accounts(
        &mut self,
        account_num: u32,
        batch_size: u32,
    ) -> Result<Vec<AccountInfo>> {
        // first get account from local
        let mut account_local = self.client.account_list()?;
        let mut available_list = vec![];
        let mut index = 0;
        let state_reader = self.client.state_reader(StateRootOption::Latest)?;
        while index < account_num {
            if let Some(account) = account_local.pop() {
                if self
                    .client
                    .account_unlock(
                        account.address,
                        self.account_password.clone(),
                        self.unlock_duration,
                    )
                    .is_ok()
                {
                    let balance = state_reader.get_balance(*account.address()).unwrap_or(None);
                    if let Some(amount) = balance {
                        if amount > 0 {
                            available_list.push(account);
                        }
                    }
                }
                index += 1;
            } else {
                break;
            }
        }

        if (available_list.len() as u32) < account_num {
            let lack_len = account_num - available_list.len() as u32;
            info!("account lack: {}", lack_len);
            // account has enough STC
            let start_balance = INITIAL_BALANCE * lack_len as u128;
            let mut balance = state_reader.get_balance(self.account_address)?;
            while balance.unwrap() < start_balance {
                std::thread::sleep(Duration::from_millis(1000));
                balance = state_reader.get_balance(self.account_address)?;
                info!(
                    "account balance is {:?}, min is: {}",
                    balance, start_balance
                );
            }
            let lack = self.create_accounts(lack_len, batch_size)?;
            //TODO fix me for reuse state_reader.
            let state_reader = self.client.state_reader(StateRootOption::Latest)?;
            for account in lack {
                let account_resource = state_reader
                    .get_account_resource(*account.address())
                    .unwrap_or(None);
                if account_resource.is_some() {
                    available_list.push(account);
                    if available_list.len() == account_num as usize {
                        break;
                    }
                }
            }
        }
        Ok(available_list)
    }

    fn create_accounts(&mut self, account_num: u32, batch_size: u32) -> Result<Vec<AccountInfo>> {
        self.unlock_account()?;
        let expiration_timestamp = self.fetch_expiration_time();
        let mut account_list = Vec::new();
        let mut i = 0;
        // let batch_size = 30;
        let mut addr_vec = vec![];
        let mut sub_account_list = vec![];
        while i < account_num {
            self.recheck_sequence_number()?;
            let account = self.client.account_create(self.account_password.clone())?;
            addr_vec.push(account.address);
            sub_account_list.push(account);
            if addr_vec.len() >= batch_size as usize {
                //submit create batch account transaction
                let txn = self.generator.generate_account_txn(
                    self.next_sequence_number,
                    self.account_address,
                    addr_vec.clone(),
                    1000000000,
                    1,
                    expiration_timestamp,
                )?;
                let result = self.submit_txn(txn, self.account_address, true);
                if matches!(result, Ok(_)) {
                    info!("account transfer submit ok.");
                } else {
                    info!("error: {:?}", result);
                }
                account_list.extend_from_slice(sub_account_list.as_slice());
                sub_account_list.clear();
                addr_vec.clear();
            }
            i += 1;
        }
        info!("{:?} accounts are created.", Vec::len(&account_list));
        Ok(account_list)
    }

    fn sequence_number<R>(&self, state_reader: &R, address: AccountAddress) -> Result<Option<u64>>
    where
        R: ChainStateReader,
    {
        let seq_number_in_pool = self.client.next_sequence_number_in_txpool(address)?;
        info!(
            "seq_number_in_pool for address {:?} is {:?}",
            address, seq_number_in_pool
        );
        let result = match seq_number_in_pool {
            Some(n) => Some(n),
            None => {
                let account_resource = state_reader.get_account_resource(address)?;
                match account_resource {
                    None => None,
                    Some(resource) => {
                        info!("read from state {:?}", resource.sequence_number());
                        Some(resource.sequence_number())
                    }
                }
            }
        };
        Ok(result)
    }

    fn stress_test(&self, accounts: Vec<AccountInfo>, round_num: u32) -> Result<()> {
        //check node status
        let sync_status = self.client.sync_status()?;
        if sync_status.is_syncing() {
            info!("node syncing, pause stress");
            return Ok(());
        }
        let state_reader = self.client.state_reader(StateRootOption::Latest)?;

        //unlock all account and get sequence
        let mut sequences = vec![];
        for account in &accounts {
            sequences.push(
                self.sequence_number(&state_reader, account.address)
                    .unwrap()
                    .unwrap(),
            );
        }
        //get  of all account
        let expiration_timestamp = self.fetch_expiration_time();
        let count = accounts.len();
        (0..round_num).for_each(|_| {
            (0..count).for_each(|index| {
                let mut j = index + 1;
                if j >= count {
                    j = 0;
                }
                let result = self.gen_and_submit_transfer_txn(
                    accounts[index].address,
                    accounts[j].address,
                    1,
                    1,
                    sequences[index],
                    false,
                    expiration_timestamp,
                );
                //handle result
                match result {
                    Ok(_) => {
                        // sequence add
                        sequences[index] += 1;
                    }
                    Err(err) => {
                        info!(
                            "Submit txn failed with error: {:?}. Try again after 500ms.",
                            err
                        );
                        std::thread::sleep(Duration::from_millis(500));
                    }
                }
            });
        });
        Ok(())
    }
}
