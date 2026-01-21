use anyhow::Result;
use starcoin_state_api::ChainStateReader as ChainStateReader1;
use starcoin_state_api::StateReaderExt as StateReaderExt1;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::multi_transaction::{MultiAccountAddress, MultiSignedUserTransaction};
use starcoin_vm2_state_api::ChainStateReader as ChainStateReader2;
use starcoin_vm2_state_api::StateReaderExt as StateReaderExt2;
use starcoin_vm2_types::account_address::AccountAddress as AccountAddress2;
use std::collections::HashMap;

pub struct StateCheck<'a, R1: ?Sized + ChainStateReader1, R2: ?Sized + ChainStateReader2> {
    state_reader1: &'a R1,
    state_reader2: &'a R2,
    sequence_cache1: HashMap<AccountAddress, u64>,
    sequence_cache2: HashMap<AccountAddress2, u64>,
}

impl<'a, R1: ?Sized + ChainStateReader1, R2: ?Sized + ChainStateReader2> StateCheck<'a, R1, R2> {
    pub fn new(state_reader1: &'a R1, state_reader2: &'a R2) -> Self {
        Self {
            state_reader1,
            state_reader2,
            sequence_cache1: HashMap::new(),
            sequence_cache2: HashMap::new(),
        }
    }

    pub fn get_next_expected_sequence(&mut self, sender: MultiAccountAddress) -> Result<u64> {
        match sender {
            MultiAccountAddress::VM1(addr) => {
                if let Some(&seq) = self.sequence_cache1.get(&addr) {
                    return Ok(seq);
                }
                let seq = self.state_reader1.get_sequence_number(addr)?;
                Ok(seq)
            }
            MultiAccountAddress::VM2(addr) => {
                if let Some(&seq) = self.sequence_cache2.get(&addr) {
                    return Ok(seq);
                }
                let seq = self.state_reader2.get_sequence_number(addr)?;
                Ok(seq)
            }
        }
    }

    pub fn check_and_accept(&mut self, txn: &MultiSignedUserTransaction) -> Result<bool> {
        let sender = txn.sender();
        let txn_seq = txn.sequence_number();
        let expected_seq = self.get_next_expected_sequence(sender)?;

        if txn_seq == expected_seq {
            match sender {
                MultiAccountAddress::VM1(addr) => {
                    self.sequence_cache1.insert(addr, txn_seq + 1);
                }
                MultiAccountAddress::VM2(addr) => {
                    self.sequence_cache2.insert(addr, txn_seq + 1);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn filter_continuous_transactions(
        &mut self,
        transactions: Vec<MultiSignedUserTransaction>,
    ) -> Result<Vec<MultiSignedUserTransaction>> {
        let mut result = Vec::with_capacity(transactions.len());

        for txn in transactions {
            match self.check_and_accept(&txn) {
                Ok(true) => result.push(txn),
                Ok(false) => {
                    continue;
                }
                Err(e) => {
                    starcoin_logger::prelude::warn!(
                        "Failed to check sequence for sender {}: {:?}",
                        txn.sender().to_hex(),
                        e
                    );
                    continue;
                }
            }
        }

        Ok(result)
    }

    pub fn clear_cache(&mut self) {
        self.sequence_cache1.clear();
        self.sequence_cache2.clear();
    }
}
