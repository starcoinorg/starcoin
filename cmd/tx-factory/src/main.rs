// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, ensure, Result};
use starcoin_account_api::AccountInfo;
use starcoin_crypto::ed25519::Ed25519PublicKey;
use starcoin_crypto::ValidCryptoMaterialStringExt;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RemoteStateReader;
use starcoin_rpc_client::RpcClient;
use starcoin_state_api::AccountStateReader;
use starcoin_tx_factory::txn_generator::MockTxnGenerator;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::association_address;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use structopt::StructOpt;
use starcoin_crypto::hash::PlainCryptoHash;

#[derive(Debug, Clone, StructOpt, Default)]
#[structopt(name = "txfactory", about = "tx generator for starcoin")]
pub struct TxFactoryOpt {
    #[structopt(long, parse(from_os_str))]
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

    #[structopt(
        long,
        short = "k",
        help = "public key(hex encoded) of address to receive balance"
    )]
    pub receiver_public_key: Option<String>,

    #[structopt(long = "stress", short = "s", help = "is stress test or not")]
    pub stress: bool,
}

const WATCH_TIMEOUT: Duration = Duration::from_secs(60);

fn get_account_or_default(
    client: &RpcClient,
    account_address: Option<AccountAddress>,
) -> Result<AccountInfo> {
    let account = match account_address {
        None => {
            let default_account = client.account_default()?;
            ensure!(
                default_account.is_some(),
                "no default account exist in the starcoin node"
            );
            default_account.unwrap()
        }
        Some(a) => match client.account_get(a)? {
            None => bail!("the specified account does not exists in the starcoin node"),
            Some(w) => w,
        },
    };

    Ok(account)
}

fn main() {
    let _logger_handler = starcoin_logger::init();
    let mut runtime = tokio_compat::runtime::Runtime::new().unwrap();
    let opts: TxFactoryOpt = TxFactoryOpt::from_args();

    let account_address = opts.account_address;
    let interval = Duration::from_millis(opts.interval);
    let account_password = opts.account_password.clone();

    let client = RpcClient::connect_ipc(opts.ipc_path, &mut runtime).expect("ipc connect success");
    let account = get_account_or_default(&client, account_address).unwrap();

    let receiver_address = opts.receiver_address.unwrap_or_else(association_address);
    let receiver_public_key = opts.receiver_public_key;
    let public_key = receiver_public_key.map(|k| {
        Ed25519PublicKey::from_encoded_string(&k).expect("public key should be hex encoded")
    });

    let is_stress = opts.stress;

    let net = client.node_info().unwrap().net;
    let node_info = client
        .node_info()
        .unwrap_or_else(|_| panic!("Failed to get node info"));
    let txn_generator = MockTxnGenerator::new(
        net.chain_id(),
        account.clone(),
        receiver_address,
        public_key,
        node_info,
    );
    let tx_mocker = TxnMocker::new(
        client,
        txn_generator,
        account.address,
        account_password,
        Duration::from_secs(60 * 10),
    );

    let mut tx_mocker = match tx_mocker {
        Ok(t) => t,
        Err(e) => {
            panic!("mocker init error: {:?}", e);
        }
    };

    let accounts = tx_mocker.create_accounts().expect("create accounts should success");

    let stopping_signal = Arc::new(AtomicBool::new(false));
    let stopping_signal_clone = stopping_signal.clone();
    ctrlc::set_handler(move || {
        stopping_signal_clone.store(true, Ordering::SeqCst);
    })
    .unwrap();
    let handle = std::thread::spawn(move || {
        while !stopping_signal.load(Ordering::SeqCst) {
            if is_stress {
                let success = tx_mocker.stress_test(accounts.clone());
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
}

impl TxnMocker {
    pub fn new(
        client: RpcClient,
        generator: MockTxnGenerator,
        account_address: AccountAddress,
        account_password: String,
        unlock_duration: Duration,
    ) -> Result<Self> {
        let state_reader = RemoteStateReader::new(&client);
        let account_state_reader = AccountStateReader::new(&state_reader);

        let account_resource = account_state_reader.get_account_resource(&account_address)?;
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
        })
    }
}

impl TxnMocker {
    fn recheck_sequence_number(&mut self) -> Result<()> {
        let seq_number_in_pool = self
            .client
            .next_sequence_number_in_txpool(self.account_address)?;

        self.next_sequence_number = match seq_number_in_pool {
            Some(n) => n,
            None => {
                let state_reader = RemoteStateReader::new(&self.client);
                let account_state_reader = AccountStateReader::new(&state_reader);

                let account_resource =
                    account_state_reader.get_account_resource(&self.account_address)?;
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

    fn is_account_exist(&mut self, account: &AccountAddress) -> bool {
        let state_reader = RemoteStateReader::new(&self.client);
        let account_state_reader = AccountStateReader::new(&state_reader);

        let account_resource =
            account_state_reader.get_account_resource(account).unwrap_or(None);
        account_resource.is_some()
    }

    fn gen_and_submit_txn(&mut self, blocking: bool) -> Result<()> {
        // check txpool, in case some txn is failed, and the sequence number will be gap-ed.
        // let seq_number_in_pool = self.client.next_sequence_number_in_txpool(self.account_address)?;
        // if let Some(n) = seq_number_in_pool {
        // }
        let raw_txn = self
            .generator
            .generate_mock_txn(self.next_sequence_number)?;
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
        let txn_hash = user_txn.crypto_hash();
        let result = self.client.submit_transaction(user_txn).and_then(|r| r);

        // increase sequence number if added in pool.
        if matches!(result, Ok(_)) {
            self.next_sequence_number += 1;
        }
        if blocking {
            self.client.watch_txn(txn_hash, Some(WATCH_TIMEOUT))?;
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

    fn gen_and_submit_transfer_txn(
        &mut self,
        sender: AccountAddress,
        receiver_address: AccountAddress,
        receiver_public_key: Option<Ed25519PublicKey>,
        amount: u128,
        sequence_number: u64,
        blocking: bool,
    ) -> Result<()> {
        let raw_txn = self.generator.generate_transfer_txn(
            sequence_number,
            sender,
            receiver_address,
            receiver_public_key,
            amount,
        )?;
        info!("prepare to sign txn, sender: {}", raw_txn.sender());

        // try unlock account
        self.client.account_unlock(
            sender,
            self.account_password.clone(),
            self.unlock_duration,
        )?;

        let user_txn = match self.client.account_sign_txn(raw_txn) {
            Err(e) => {
                // sign txn fail, we should unlock again
                //self.account_unlock_time = None;
                return Err(e);
            }
            Ok(txn) => txn,
        };
        info!(
            "prepare to submit txn, sender:{},seq:{}",
            user_txn.sender(),
            user_txn.sequence_number(),
        );
        let txn_hash = user_txn.crypto_hash();
        let result = self.client.submit_transaction(user_txn).and_then(|r| r);

        if matches!(result, Ok(_)) && blocking {
            self.client.watch_txn(txn_hash, Some(WATCH_TIMEOUT))?;
        }
        result
    }

    fn create_accounts(&mut self) -> Result<Vec<AccountInfo>> {
        self.unlock_account()?;

        let mut account_list = Vec::new();
        for _i in 0..10 {
            self.recheck_sequence_number()?;
            let account = self.client.account_create(self.account_password.clone())?;
            let result = self.gen_and_submit_transfer_txn(
                self.account_address,
                account.address.clone(),
                account.public_key.as_single(),
                100000,
                self.next_sequence_number,
                true,
            );
            if matches!(result, Ok(_)) {
                if self.is_account_exist(&account.address) {
                    account_list.push(account);
                } else {
                    info!("create account failed, watch timeout?");
                }
            }
        }
        Ok(account_list)
    }

    fn transfer_to_accounts(&mut self, accounts: &Vec<AccountInfo>) -> Result<()> {
        self.unlock_account()?;
        self.recheck_sequence_number()?;
        for account in accounts {
            let result = self.gen_and_submit_transfer_txn(
                self.account_address,
                account.address.clone(),
                account.public_key.as_single(),
                100000,
                self.next_sequence_number,
                false,
            );
            if matches!(result, Ok(_)) {
                self.next_sequence_number += 1;
            }
        }
        Ok(())
    }

    fn stress_test(&mut self, accounts: Vec<AccountInfo>) -> Result<()> {
        self.transfer_to_accounts(&accounts)?;
        for account in &accounts {
            let seq = self.sequence_number(account.address.clone())?;
            if seq.is_some() {
                let mut seq_num= seq.unwrap();
                for receiver in &accounts {
                    if account.address != receiver.address {
                        let result = self.gen_and_submit_transfer_txn(
                            account.address.clone(),
                            receiver.address.clone(),
                            receiver.public_key.as_single(),
                            10,
                            seq_num,
                            false,
                        );
                        if matches!(result, Ok(_)) {
                            seq_num += 1;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn sequence_number(&mut self, address: AccountAddress) -> Result<Option<u64>> {
        let seq_number_in_pool = self
            .client
            .next_sequence_number_in_txpool(address)?;

        let result = match seq_number_in_pool {
            Some(n) => Some(n),
            None => {
                let state_reader = RemoteStateReader::new(&self.client);
                let account_state_reader = AccountStateReader::new(&state_reader);

                let account_resource =
                    account_state_reader.get_account_resource(&address)?;
                if account_resource.is_none() {
                    None
                } else {
                    Some(account_resource.unwrap().sequence_number())
                }
            }
        };
        Ok(result)
    }
}
