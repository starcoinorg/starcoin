use anyhow::{bail, format_err, Result};
use starcoin_account_api::AccountInfo;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::{error, info, warn};
use starcoin_rpc_api::types::FactoryAction;
use starcoin_rpc_client::{RpcClient, StateRootOption};
use starcoin_state_api::{ChainStateReader, StateReaderExt};
use starcoin_types::{
    account_address::AccountAddress, sync_status::SyncStatus, transaction::RawUserTransaction,
};
use starcoin_vm2_account_api::AccountInfo as AccountInfo2;
use starcoin_vm2_vm_types::{
    account_address::AccountAddress as AccountAddress2, state_view::StateReaderExt as _,
    transaction::RawUserTransaction as RawUserTransaction2,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::txn_generator::MockTxnGenerator;

pub const INITIAL_BALANCE: u128 = 1_000_000_000;

pub struct TxnMocker {
    client: RpcClient,
    generator: MockTxnGenerator,
    account_address: AccountAddress,
    account_address2: AccountAddress2,
    account_password: String,
    account_password2: String,
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
        account_address2: AccountAddress2,
        account_password2: String,
        unlock_duration: Duration,
        watch_timeout: u32,
    ) -> Result<Self> {
        let state_reader = client.state_reader2(StateRootOption::Latest)?;

        let account_resource = state_reader.get_account_resource(account_address2)?;

        let mut next_sequence_number = account_resource.sequence_number();
        // if txpool already has some future txn, use the sequence number after that.
        let seq_number_in_txpool = client.next_sequence_number_in_txpool2(account_address2)?;
        if let Some(n) = seq_number_in_txpool {
            if n > next_sequence_number {
                next_sequence_number = n;
            }
        }
        Ok(Self {
            client,
            generator,
            account_address,
            account_address2,
            account_password,
            account_password2,
            unlock_duration,
            account_unlock_time: None,
            next_sequence_number,
            watch_timeout,
        })
    }
}

impl TxnMocker {
    fn fetch_expiration_time(&self) -> u64 {
        let now = SystemTime::now();
        now.duration_since(UNIX_EPOCH)
            .expect("time error")
            .as_secs()
            + 180 // 3 minutes
                  // let node_info = self
                  //     .client
                  //     .node_info()
                  //     .expect("node_info() should not failed");
                  // node_info.now_seconds + DEFAULT_EXPIRATION_TIME
    }
    pub fn get_factory_status(&self) -> bool {
        self.client
            .debug_txfactory_status(FactoryAction::Status)
            .unwrap()
    }

    pub fn recheck_sequence_number2(&mut self) -> Result<()> {
        let seq_number_in_pool = self
            .client
            .next_sequence_number_in_txpool2(self.account_address2)?;

        self.next_sequence_number = match seq_number_in_pool {
            Some(n) => n,
            None => {
                let state_reader = self.client.state_reader2(StateRootOption::Latest)?;
                let account_resource = state_reader.get_account_resource(self.account_address2)?;
                account_resource.sequence_number()
            }
        };
        Ok(())
    }

    pub fn recheck_sequence_number(&mut self) -> Result<()> {
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

    pub fn gen_and_submit_txn2(&mut self, blocking: bool) -> Result<HashValue> {
        let expiration_timestamp = self.fetch_expiration_time();
        let raw_txn = self
            .generator
            .generate_mock_txn2(self.next_sequence_number, expiration_timestamp)?;
        info!("prepare to sign txn, sender: {}", raw_txn.sender());

        self.unlock_account()?;

        let user_txn = match self.client.account_sign_txn2(raw_txn) {
            Err(e) => {
                // sign txn fail, we should unlock again
                self.account_unlock_time = None;
                return Err(e);
            }
            Ok(txn) => txn,
        };
        info!(
            "prepare to submit txn, sender:{},seq:{},id:{}",
            user_txn.sender(),
            user_txn.sequence_number(),
            user_txn.id(),
        );
        let txn_hash = user_txn.id();
        let result = self.client.submit_transaction2(user_txn);

        // increase sequence number if added in pool.
        if result.is_ok() {
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

    pub fn gen_and_submit_txn(&mut self, blocking: bool) -> Result<HashValue> {
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
            "prepare to submit txn, sender:{},seq:{},id:{}",
            user_txn.sender(),
            user_txn.sequence_number(),
            user_txn.id(),
        );
        let txn_hash = user_txn.id();
        let result = self.client.submit_transaction(user_txn);

        // increase sequence number if added in pool.
        if result.is_ok() {
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

    fn unlock_account2(&mut self) -> Result<()> {
        let unlock_time = self.account_unlock_time;
        match unlock_time {
            Some(t) if t + self.unlock_duration > Instant::now() => {}
            _ => {
                // reset first just in case account_unlock fail
                self.account_unlock_time = None;

                let new_unlock_time = Instant::now();
                // try unlock account
                self.client.account_unlock2(
                    self.account_address2,
                    self.account_password2.clone(),
                    self.unlock_duration,
                )?;

                self.account_unlock_time = Some(new_unlock_time);
            }
        }
        Ok(())
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

    fn submit_transaction_in_batch2(
        &self,
        txns: Vec<(AccountAddress2, Vec<RawUserTransaction2>)>,
        blocking: bool,
    ) -> Result<()> {
        info!("going to unlock accounts");
        self.client.account_unlock_in_batch2(
            txns.iter()
                .map(|(sender, _)| (*sender, self.account_password2.clone()))
                .collect(),
            self.unlock_duration,
        )?;

        let signed_transactions = self.client.account_sign_txn_in_batch2(
            txns.iter()
                .flat_map(|(_, raw_txns)| raw_txns.clone())
                .collect(),
        )?;

        let hashes = self.client.submit_transactions2(signed_transactions)?;
        info!("submitted {} txns", hashes.len());

        if blocking {
            for hash in hashes {
                self.client
                    .watch_txn(hash, Some(Duration::from_secs(self.watch_timeout as u64)))?;
            }
        }
        Ok(())
    }

    fn submit_transaction_in_batch(
        &self,
        txns: Vec<(AccountAddress, Vec<RawUserTransaction>)>,
        blocking: bool,
    ) -> Result<()> {
        info!("going to unlock accounts");
        self.client.account_unlock_in_batch(
            txns.iter()
                .map(|(sender, _)| (*sender, self.account_password.clone()))
                .collect(),
            self.unlock_duration,
        )?;

        let signed_transactions = self.client.account_sign_txn_in_batch(
            txns.iter()
                .flat_map(|(_, raw_txns)| raw_txns.clone())
                .collect(),
        )?;

        let hashes = self.client.submit_transactions(signed_transactions)?;
        info!("submitted {} txns", hashes.len());

        if blocking {
            for hash in hashes {
                self.client
                    .watch_txn(hash, Some(Duration::from_secs(self.watch_timeout as u64)))?;
            }
        }
        Ok(())
    }

    fn submit_txn2(
        &self,
        raw_txn: RawUserTransaction2,
        sender: AccountAddress2,
        blocking: bool,
    ) -> Result<HashValue> {
        // try unlock account
        self.client.account_unlock2(
            sender,
            self.account_password2.clone(),
            self.unlock_duration,
        )?;

        let user_txn = match self.client.account_sign_txn2(raw_txn) {
            Err(e) => {
                return Err(e);
            }
            Ok(txn) => txn,
        };
        info!(
            "prepare to submit txn, sender:{},seq:{},id:{}",
            user_txn.sender(),
            user_txn.sequence_number(),
            user_txn.id(),
        );
        let txn_hash = user_txn.id();
        let result = self.client.submit_transaction2(user_txn);

        if result.is_ok() && blocking {
            self.client.watch_txn(
                txn_hash,
                Some(Duration::from_secs(self.watch_timeout as u64)),
            )?;
        }
        result
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
            "prepare to submit txn, sender:{},seq:{},id:{}",
            user_txn.sender(),
            user_txn.sequence_number(),
            user_txn.id(),
        );
        let txn_hash = user_txn.id();
        let result = self.client.submit_transaction(user_txn);

        if result.is_ok() && blocking {
            self.client.watch_txn(
                txn_hash,
                Some(Duration::from_secs(self.watch_timeout as u64)),
            )?;
        }
        result
    }

    #[allow(dead_code)]
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

    pub fn get_or_create_accounts2(
        &mut self,
        account_num: u32,
        batch_size: u32,
    ) -> Result<Vec<AccountInfo2>> {
        // first get account from local
        let mut account_local = self.client.account_list2()?;
        let mut available_list = vec![];
        let mut index = 0;
        let state_reader = self.client.state_reader2(StateRootOption::Latest)?;
        while index < account_num {
            if let Some(account) = account_local.pop() {
                if self
                    .client
                    .account_unlock2(
                        account.address,
                        self.account_password2.clone(),
                        self.unlock_duration,
                    )
                    .is_ok()
                {
                    let balance = state_reader
                        .get_balance2(*account.address())
                        .unwrap_or(None);
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
            let mut balance = state_reader.get_balance2(self.account_address2)?;
            while balance.unwrap() < start_balance {
                std::thread::sleep(Duration::from_millis(1000));
                balance = state_reader.get_balance2(self.account_address2)?;
                info!(
                    "account balance is {:?}, min is: {}",
                    balance, start_balance
                );
            }
            let lack = self.create_accounts2(lack_len, batch_size)?;
            //TODO fix me for reuse state_reader.
            let state_reader = self.client.state_reader2(StateRootOption::Latest)?;
            for account in lack {
                match state_reader.get_account_resource(*account.address()) {
                    Ok(_resournce) => {
                        available_list.push(account);
                        if available_list.len() == account_num as usize {
                            break;
                        }
                    }
                    Err(e) => warn!("get account resource error: {e}"),
                }
            }
        }
        Ok(available_list)
    }

    pub fn get_or_create_accounts(
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

    pub fn create_accounts2(
        &mut self,
        account_num: u32,
        batch_size: u32,
    ) -> Result<Vec<AccountInfo2>> {
        self.unlock_account2()?;
        let expiration_timestamp = self.fetch_expiration_time();
        let mut account_list = Vec::new();
        let mut i = 0;
        // let batch_size = 30;
        let mut addr_vec = vec![];
        let mut sub_account_list = vec![];
        while i < account_num {
            self.recheck_sequence_number2()?;
            let account = match self.client.account_create2(self.account_password2.clone()) {
                Ok(account) => account,
                Err(e) => {
                    error!("create account error: {}", e);
                    continue;
                }
            };
            addr_vec.push(account.address);
            sub_account_list.push(account);
            if addr_vec.len() >= batch_size as usize {
                //submit create batch account transaction
                //)?;
                let txn = self.generator.generate_account_txn2(
                    self.next_sequence_number,
                    self.account_address2,
                    addr_vec.clone(),
                    1,
                    1,
                    expiration_timestamp,
                )?;
                let result = self.submit_txn2(txn, self.account_address2, true);
                if result.is_ok() {
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

        if !addr_vec.is_empty() {
            self.recheck_sequence_number()?;
            self.unlock_account()?;
            let txn = self.generator.generate_account_txn2(
                self.next_sequence_number,
                self.account_address2,
                addr_vec.clone(),
                1,
                10000,
                expiration_timestamp,
            )?;
            let result = self.submit_txn2(txn, self.account_address2, true);
            if result.is_ok() {
                info!("account transfer submit ok.");
            } else {
                info!("error: {:?}", result);
            }
            account_list.extend_from_slice(sub_account_list.as_slice());
        }

        info!("{:?} accounts are created.", Vec::len(&account_list));
        Ok(account_list)
    }

    pub fn create_accounts(
        &mut self,
        account_num: u32,
        batch_size: u32,
    ) -> Result<Vec<AccountInfo>> {
        self.unlock_account()?;
        let expiration_timestamp = self.fetch_expiration_time();
        let mut account_list = Vec::new();
        let mut i = 0;
        // let batch_size = 30;
        let mut addr_vec = vec![];
        let mut sub_account_list = vec![];
        while i < account_num {
            self.recheck_sequence_number()?;
            let account = match self.client.account_create(self.account_password.clone()) {
                Ok(account) => account,
                Err(e) => {
                    error!("create account error: {}", e);
                    continue;
                }
            };
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
                if result.is_ok() {
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

        if !addr_vec.is_empty() {
            self.recheck_sequence_number()?;
            self.unlock_account()?;
            let txn = self.generator.generate_account_txn(
                self.next_sequence_number,
                self.account_address,
                addr_vec.clone(),
                1,
                10000,
                expiration_timestamp,
            )?;
            let result = self.submit_txn(txn, self.account_address, true);
            if result.is_ok() {
                info!("account transfer submit ok.");
            } else {
                info!("error: {:?}", result);
            }
            account_list.extend_from_slice(sub_account_list.as_slice());
        }

        info!("{:?} accounts are created.", Vec::len(&account_list));
        Ok(account_list)
    }

    fn next_sequence_number_in_batch2(
        &self,
        addresses: Vec<AccountAddress2>,
    ) -> Result<Vec<(AccountAddress2, Option<u64>)>> {
        let seq_numbers = self
            .client
            .next_sequence_number_in_batch2(addresses)?
            .ok_or_else(|| format_err!("next_sequence_number_in_batch error"))?;
        Ok(seq_numbers
            .into_iter()
            .map(|(address, seq_number)| match seq_number {
                Some(seq_number) => (address, Some(seq_number)),
                None => {
                    let state_reader = self
                        .client
                        .state_reader2(StateRootOption::Latest)
                        .expect("state_reader error");
                    let account_resource = state_reader
                        .get_account_resource(address)
                        .expect("get_account_resource error");
                    let seq = account_resource.sequence_number();
                    (address, Some(seq))
                }
            })
            .collect())
    }

    fn next_sequence_number_in_batch(
        &self,
        addresses: Vec<AccountAddress>,
    ) -> Result<Vec<(AccountAddress, Option<u64>)>> {
        let seq_numbers = self
            .client
            .next_sequence_number_in_batch(addresses)?
            .ok_or_else(|| format_err!("next_sequence_number_in_batch error"))?;
        Ok(seq_numbers
            .into_iter()
            .map(|(address, seq_number)| match seq_number {
                Some(seq_number) => (address, Some(seq_number)),
                None => {
                    let state_reader = self
                        .client
                        .state_reader(StateRootOption::Latest)
                        .expect("state_reader error");
                    let account_resource = state_reader
                        .get_account_resource(address)
                        .expect("get_account_resource error");
                    let seq = account_resource.map(|resource| resource.sequence_number());
                    (address, seq)
                }
            })
            .collect())
    }

    #[allow(dead_code)]
    fn sequence_number<R>(&self, _state_reader: &R, address: AccountAddress) -> Result<Option<u64>>
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
                let state_reader = self.client.state_reader(StateRootOption::Latest)?;
                let account_resource = state_reader.get_account_resource(address)?;
                account_resource.map(|resource| resource.sequence_number())
            }
        };
        Ok(result)
    }

    fn send_and_receive2(
        &self,
        senders: Vec<(AccountAddress2, Option<u64>)>,
        receivers: Vec<AccountAddress2>,
        amount: u128,
        round_num: u64,
    ) -> Result<()> {
        if receivers.len() < senders.len() {
            bail!("receivers len {} is less than senders len {}, the account number should be better even", receivers.len(), senders.len());
        }
        let mut transactions = Vec::new();
        let mut sender_transactions = Vec::new();
        for (index, (sender_address, sequence_op)) in senders.iter().enumerate() {
            let seq = match sequence_op {
                Some(seq) => seq,
                None => {
                    error!("address {:?} seq is none", sender_address);
                    continue;
                }
            };

            for i in 0..round_num {
                let txn = self.generator.generate_transfer_txn2(
                    *seq + i,
                    *sender_address,
                    receivers[index],
                    amount,
                    1,
                    self.fetch_expiration_time(),
                )?;
                transactions.push(txn);
            }
            sender_transactions.push((*sender_address, transactions.clone()));
            transactions.clear();
        }

        self.submit_transaction_in_batch2(sender_transactions, false)
    }

    fn send_and_receive(
        &self,
        senders: Vec<(AccountAddress, Option<u64>)>,
        receivers: Vec<AccountAddress>,
        amount: u128,
        round_num: u64,
    ) -> Result<()> {
        if receivers.len() < senders.len() {
            bail!("receivers len {} is less than senders len {}, the account number should be better even", receivers.len(), senders.len());
        }
        let mut transactions = Vec::new();
        let mut sender_transactions = Vec::new();
        for (index, (sender_address, sequence_op)) in senders.iter().enumerate() {
            let seq = match sequence_op {
                Some(seq) => seq,
                None => {
                    error!("address {:?} seq is none", sender_address);
                    continue;
                }
            };

            for i in 0..round_num {
                let txn = self.generator.generate_transfer_txn(
                    *seq + i,
                    *sender_address,
                    receivers[index],
                    amount,
                    1,
                    self.fetch_expiration_time(),
                )?;
                transactions.push(txn);
            }
            sender_transactions.push((*sender_address, transactions.clone()));
            transactions.clear();
        }

        self.submit_transaction_in_batch(sender_transactions, false)
    }

    pub fn stress_test2(
        &self,
        accounts: Vec<AccountInfo2>,
        round_num: u32,
        interval: Duration,
        transfer_account_size: usize,
    ) -> Result<()> {
        //check node status
        let sync_status: SyncStatus = self.client.sync_status()?.into();
        if sync_status.is_syncing() {
            info!("node syncing, pause stress");
            return Ok(());
        }

        //unlock all account and get sequence
        let sequences =
            self.next_sequence_number_in_batch2(accounts.iter().map(|a| a.address).collect())?;

        for addresses in sequences.chunks(transfer_account_size) {
            let mid = addresses.len() / 2;
            let senders = &addresses[..mid];
            let receivers = &addresses[mid..];
            self.send_and_receive2(
                senders.to_vec(),
                receivers
                    .iter()
                    .copied()
                    .map(|(address, _)| address)
                    .collect(),
                1,
                round_num as u64,
            )?;

            std::thread::sleep(interval);
        }

        for addresses in sequences.rchunks(transfer_account_size) {
            let mid = addresses.len() / 2;
            let senders = &addresses[..mid];
            let receivers = &addresses[mid..];
            self.send_and_receive2(
                senders.to_vec(),
                receivers
                    .iter()
                    .copied()
                    .map(|(address, _)| address)
                    .collect(),
                1,
                round_num as u64,
            )?;

            std::thread::sleep(interval);
        }
        Ok(())
    }

    pub fn stress_test(
        &self,
        accounts: Vec<AccountInfo>,
        round_num: u32,
        interval: Duration,
        transfer_account_size: usize,
    ) -> Result<()> {
        //check node status
        let sync_status: SyncStatus = self.client.sync_status()?.into();
        if sync_status.is_syncing() {
            info!("node syncing, pause stress");
            return Ok(());
        }

        //unlock all account and get sequence
        let sequences =
            self.next_sequence_number_in_batch(accounts.iter().map(|a| a.address).collect())?;

        for addresses in sequences.chunks(transfer_account_size) {
            let mid = addresses.len() / 2;
            let senders = &addresses[..mid];
            let receivers = &addresses[mid..];
            self.send_and_receive(
                senders.to_vec(),
                receivers
                    .iter()
                    .copied()
                    .map(|(address, _)| address)
                    .collect(),
                1,
                round_num as u64,
            )?;

            std::thread::sleep(interval);
        }

        for addresses in sequences.rchunks(transfer_account_size) {
            let mid = addresses.len() / 2;
            let senders = &addresses[..mid];
            let receivers = &addresses[mid..];
            self.send_and_receive(
                senders.to_vec(),
                receivers
                    .iter()
                    .copied()
                    .map(|(address, _)| address)
                    .collect(),
                1,
                round_num as u64,
            )?;

            std::thread::sleep(interval);
        }
        Ok(())
    }
}
