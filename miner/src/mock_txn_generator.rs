use anyhow::Result;
use executor::mock_executor::mock_transfer_txn_with_seq_number;
use logger::prelude::*;
use rand;
use rand::Rng;
use std::convert::TryInto;
// use std::sync::RwLock;
use traits::ChainStateReader;
use types::{
    access_path::AccessPath, account_address::AccountAddress, account_config::AccountResource,
    transaction::Transaction,
};

pub(crate) struct MockTxnGenerator {
    account_address: AccountAddress,
    // last_seen_seq_number: RwLock<Option<u64>>,
}

impl MockTxnGenerator {
    pub fn new(account_address: AccountAddress) -> Self {
        MockTxnGenerator {
            account_address,
            // last_seen_seq_number: RwLock::new(None),
        }
    }

    pub fn generate_mock_txn(&self, state_db: &dyn ChainStateReader) -> Result<Transaction> {
        // account already exists
        let ap = AccessPath::new_for_account(self.account_address);
        let account_resource: Option<AccountResource> = match state_db.get(&ap)? {
            None => None,
            Some(b) => Some(b.try_into()?),
        };
        debug!(
            "mocked_account: address:{},balance:{:?},seq:{:?}",
            self.account_address,
            account_resource.as_ref().map(|r| r.balance()),
            account_resource.as_ref().map(|r| r.sequence_number())
        );
        // let last_seen_number = self.last_seen_seq_number.read().unwrap().clone();
        // if let Some(r) = &account_resource {
        //     match last_seen_number {
        //         None => {
        //             *self.last_seen_seq_number.write().unwrap() = Some(r.sequence_number());
        //         }
        //         Some(seen) => {
        //             if r.sequence_number() > seen {
        //                 *self.last_seen_seq_number.write().unwrap() = Some(r.sequence_number());
        //             } else {
        //                 bail!(
        //                     "wait prev txn(account: {}, seq: {}) to be minted",
        //                     self.account_address,
        //                     seen
        //                 );
        //             }
        //         }
        //     }
        // }

        match account_resource {
            Some(r) if r.balance() > 3000 => {
                // add some randomness.
                let mut rng = rand::thread_rng();
                let amount_to_transfer = rng.gen_range(1000u64, 5000u64);
                // return money back to faucet account
                let transfer_txn = mock_transfer_txn_with_seq_number(
                    r.sequence_number(),
                    self.account_address,
                    AccountAddress::DEFAULT,
                    amount_to_transfer,
                );
                Ok(transfer_txn)
            }
            _ => {
                // use faucet to get money
                let faucet_account_address = AccountAddress::DEFAULT;
                let ap = AccessPath::new_for_account(faucet_account_address);
                let faucet_account_resource: AccountResource = state_db
                    .get(&ap)?
                    .expect("faucet account must exists")
                    .try_into()?;
                let cur_sequence_number = faucet_account_resource.sequence_number();
                // add some randomness too.
                let mut rng = rand::thread_rng();
                let amount_to_transfer = rng.gen_range(5000u64, 10000u64);

                // transfer will create account if it not exists.
                let transfer_txn = mock_transfer_txn_with_seq_number(
                    cur_sequence_number,
                    AccountAddress::DEFAULT,
                    self.account_address,
                    amount_to_transfer,
                );
                Ok(transfer_txn)
            }
        }
    }
}
