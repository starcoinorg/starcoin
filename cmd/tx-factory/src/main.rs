// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, ensure, Result};
use ctrlc;
use starcoin_crypto::ed25519::Ed25519PublicKey;
use starcoin_crypto::ValidKeyStringExt;
use starcoin_executor::executor::Executor;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RemoteStateReader;
use starcoin_rpc_client::RpcClient;
use starcoin_state_api::AccountStateReader;
use starcoin_tx_factory::txn_generator::MockTxnGenerator;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::association_address;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_wallet_api::WalletAccount;
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
        help = "public key(hex encoded) of address to receive balance, default to none"
    )]
    pub receiver_public_key: Option<String>,
}

fn get_wallet_account(
    client: &RpcClient,
    account_address: Option<AccountAddress>,
) -> Result<WalletAccount> {
    let account = match account_address {
        None => {
            let all_account = client.wallet_list()?;
            let default_account = all_account.into_iter().find(|w| w.is_default);

            ensure!(
                default_account.is_some(),
                "no default account exist in the starcoin node"
            );
            default_account.unwrap()
        }
        Some(a) => match client.wallet_get(a)? {
            None => bail!("the specified account does not exists in the starcoin node"),
            Some(w) => w,
        },
    };

    Ok(account)
}

fn main() {
    let _logger_handler = starcoin_logger::init();
    let opts: TxFactoryOpt = TxFactoryOpt::from_args();

    let account_address = opts.account_address;
    let interval = Duration::from_millis(opts.interval);
    let account_password = opts.account_password.clone();

    let client = RpcClient::connect_ipc(opts.ipc_path).expect("ipc connect success");
    let account = get_wallet_account(&client, account_address).unwrap();

    let receiver_address = opts.receiver_address.unwrap_or(association_address());
    let receiver_public_key = opts.receiver_public_key.clone();
    let receiver_auth_key_prefix = receiver_public_key
        .map(|k| {
            let k = Ed25519PublicKey::from_encoded_string(&k)
                .expect("public key should be hex encoded");
            AuthenticationKey::ed25519(&k).prefix().to_vec()
        })
        .unwrap_or_default();

    let txn_generator =
        MockTxnGenerator::new(account.clone(), receiver_address, receiver_auth_key_prefix);
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

    let stopping_signal = Arc::new(AtomicBool::new(false));
    let stopping_signal_clone = stopping_signal.clone();
    ctrlc::set_handler(move || {
        stopping_signal_clone.store(true, Ordering::SeqCst);
    })
    .unwrap();
    let handle = std::thread::spawn(move || {
        while !stopping_signal.load(Ordering::SeqCst) {
            let success = tx_mocker.gen_and_submit_txn();
            match success {
                Ok(s) => {
                    warn!("submit status: {}", s);
                    // if txn is rejected, recheck sequence number, and start over
                    if !s {
                        tx_mocker.recheck_sequence_number();
                    }
                }
                Err(e) => {
                    error!("fail to generate/submit mock txn, err: {:?}", &e);
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
                let state_reader = RemoteStateReader::new(&client);
                let account_state_reader = AccountStateReader::new(&state_reader);

                let account_resource =
                    account_state_reader.get_account_resource(&account_address)?;
                if account_resource.is_none() {
                    bail!("account {} not exists, please faucet it", account_address);
                }
                account_resource.unwrap().sequence_number()
            }
        };
        Ok(())
    }
    fn gen_and_submit_txn(&mut self) -> Result<bool> {
        // check txpool, in case some txn is failed, and the sequence number will be gap-ed.
        // let seq_number_in_pool = self.client.next_sequence_number_in_txpool(self.account_address)?;
        // if let Some(n) = seq_number_in_pool {
        // }
        let raw_txn = self
            .generator
            .generate_mock_txn::<Executor>(self.next_sequence_number)?;
        info!("prepare to sign txn, sender: {}", raw_txn.sender());

        let unlock_time = self.account_unlock_time.clone();
        match unlock_time {
            Some(t) if t + self.unlock_duration > Instant::now() => {}
            _ => {
                // reset first just in case account_unlock fail
                self.account_unlock_time = None;

                let new_unlock_time = Instant::now();
                // try unlock account
                self.client.wallet_unlock(
                    self.account_address,
                    self.account_password.clone(),
                    self.unlock_duration,
                )?;

                self.account_unlock_time = Some(new_unlock_time);
            }
        }

        let user_txn = match self.client.wallet_sign_txn(raw_txn) {
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
        let result = self.client.submit_transaction(user_txn);

        // increase sequence number if added in pool.
        if matches!(result, Ok(t) if t) {
            self.next_sequence_number += 1;
        }
        result
    }
}
