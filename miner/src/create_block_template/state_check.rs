use anyhow::Result;
use starcoin_state_api::{
    AccountStateReader as AccountStateReader1, ChainStateReader as ChainStateReader1,
};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::SignedUserTransaction;
use starcoin_vm2_state_api::{
    AccountStateReader as AccountStateReader2, ChainStateReader as ChainStateReader2,
};
use starcoin_vm2_types::account_address::AccountAddress as AccountAddress2;
use starcoin_vm2_vm_types::transaction::SignedUserTransaction as SignedUserTransaction2;
use std::collections::HashMap;

pub struct StateCheck<'a, R: ChainStateReader1> {
    state_reader: &'a R,
    sequence_cache: HashMap<AccountAddress, u64>,
}

impl<'a, R: ChainStateReader1> StateCheck<'a, R> {
    pub fn new(state_reader: &'a R) -> Self {
        Self {
            state_reader,
            sequence_cache: HashMap::new(),
        }
    }

    pub fn get_sequence_number(&self, sender: AccountAddress) -> Result<u64> {
        let account_reader = AccountStateReader1::new(self.state_reader);
        account_reader.get_sequence_number(sender)
    }

    pub fn get_next_expected_sequence(&mut self, sender: AccountAddress) -> Result<u64> {
        if let Some(&seq) = self.sequence_cache.get(&sender) {
            return Ok(seq);
        }

        let seq = self.get_sequence_number(sender)?;
        Ok(seq)
    }

    pub fn is_sequence_continuous(&mut self, txn: &SignedUserTransaction) -> Result<bool> {
        let sender = txn.sender();
        let txn_seq = txn.sequence_number();
        let expected_seq = self.get_next_expected_sequence(sender)?;

        Ok(txn_seq == expected_seq)
    }

    pub fn check_and_accept(&mut self, txn: &SignedUserTransaction) -> Result<bool> {
        let sender = txn.sender();
        let txn_seq = txn.sequence_number();
        let expected_seq = self.get_next_expected_sequence(sender)?;

        if txn_seq == expected_seq {
            self.sequence_cache.insert(sender, txn_seq + 1);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn filter_continuous_transactions(
        &mut self,
        transactions: Vec<SignedUserTransaction>,
    ) -> Result<Vec<SignedUserTransaction>> {
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
                        txn.sender(),
                        e
                    );
                    continue;
                }
            }
        }

        Ok(result)
    }

    pub fn reset_sender_cache(&mut self, sender: &AccountAddress) {
        self.sequence_cache.remove(sender);
    }

    pub fn clear_cache(&mut self) {
        self.sequence_cache.clear();
    }

    pub fn check_sequence_detail(
        &mut self,
        txn: &SignedUserTransaction,
    ) -> Result<SequenceCheckResult<AccountAddress>> {
        let sender = txn.sender();
        let txn_seq = txn.sequence_number();
        let expected_seq = self.get_next_expected_sequence(sender)?;

        if txn_seq == expected_seq {
            Ok(SequenceCheckResult::Continuous {
                sender,
                sequence: txn_seq,
            })
        } else if txn_seq < expected_seq {
            Ok(SequenceCheckResult::TooLow {
                sender,
                actual: txn_seq,
                expected: expected_seq,
            })
        } else {
            Ok(SequenceCheckResult::Gap {
                sender,
                actual: txn_seq,
                expected: expected_seq,
            })
        }
    }
}

pub struct StateCheck2<'a, R: ChainStateReader2> {
    state_reader: &'a R,
    sequence_cache: HashMap<AccountAddress2, u64>,
}

impl<'a, R: ChainStateReader2> StateCheck2<'a, R> {
    pub fn new(state_reader: &'a R) -> Self {
        Self {
            state_reader,
            sequence_cache: HashMap::new(),
        }
    }

    pub fn get_sequence_number(&self, sender: AccountAddress2) -> Result<u64> {
        let account_reader = AccountStateReader2::new(self.state_reader);
        account_reader.get_sequence_number(sender)
    }

    pub fn get_next_expected_sequence(&mut self, sender: AccountAddress2) -> Result<u64> {
        if let Some(&seq) = self.sequence_cache.get(&sender) {
            return Ok(seq);
        }

        let seq = self.get_sequence_number(sender)?;
        Ok(seq)
    }

    pub fn is_sequence_continuous(&mut self, txn: &SignedUserTransaction2) -> Result<bool> {
        let sender = txn.sender();
        let txn_seq = txn.sequence_number();
        let expected_seq = self.get_next_expected_sequence(sender)?;

        Ok(txn_seq == expected_seq)
    }

    pub fn check_and_accept(&mut self, txn: &SignedUserTransaction2) -> Result<bool> {
        let sender = txn.sender();
        let txn_seq = txn.sequence_number();
        let expected_seq = self.get_next_expected_sequence(sender)?;

        if txn_seq == expected_seq {
            self.sequence_cache.insert(sender, txn_seq + 1);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn filter_continuous_transactions(
        &mut self,
        transactions: Vec<SignedUserTransaction2>,
    ) -> Result<Vec<SignedUserTransaction2>> {
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
                        txn.sender(),
                        e
                    );
                    continue;
                }
            }
        }

        Ok(result)
    }

    pub fn reset_sender_cache(&mut self, sender: &AccountAddress2) {
        self.sequence_cache.remove(sender);
    }

    pub fn clear_cache(&mut self) {
        self.sequence_cache.clear();
    }

    pub fn check_sequence_detail(
        &mut self,
        txn: &SignedUserTransaction2,
    ) -> Result<SequenceCheckResult<AccountAddress2>> {
        let sender = txn.sender();
        let txn_seq = txn.sequence_number();
        let expected_seq = self.get_next_expected_sequence(sender)?;

        if txn_seq == expected_seq {
            Ok(SequenceCheckResult::Continuous {
                sender,
                sequence: txn_seq,
            })
        } else if txn_seq < expected_seq {
            Ok(SequenceCheckResult::TooLow {
                sender,
                actual: txn_seq,
                expected: expected_seq,
            })
        } else {
            Ok(SequenceCheckResult::Gap {
                sender,
                actual: txn_seq,
                expected: expected_seq,
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SequenceCheckResult<A> {
    Continuous {
        sender: A,
        sequence: u64,
    },
    TooLow {
        sender: A,
        actual: u64,
        expected: u64,
    },
    Gap {
        sender: A,
        actual: u64,
        expected: u64,
    },
}

impl<A: Copy> SequenceCheckResult<A> {
    pub fn is_continuous(&self) -> bool {
        matches!(self, SequenceCheckResult::Continuous { .. })
    }

    pub fn sender(&self) -> A {
        match self {
            SequenceCheckResult::Continuous { sender, .. } => *sender,
            SequenceCheckResult::TooLow { sender, .. } => *sender,
            SequenceCheckResult::Gap { sender, .. } => *sender,
        }
    }
}
