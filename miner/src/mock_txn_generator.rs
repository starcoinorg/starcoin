use anyhow::Result;
use executor::mock_executor::mock_transfer_txn_with_seq_number;
use logger::prelude::*;
use rand;
use rand::{Rng, SeedableRng};
use std::convert::TryInto;
// use std::sync::RwLock;
use crypto::ed25519::*;
use crypto::test_utils::KeyPair;
use executor::TransactionExecutor;
use starcoin_crypto::Uniform;
use traits::ChainStateReader;
use types::{
    access_path::AccessPath, account_address::AccountAddress, account_address::AuthenticationKey,
    account_config::association_address, account_config::AccountResource, transaction::Transaction,
};

type AccountKeyPair = KeyPair<Ed25519PrivateKey, Ed25519PublicKey>;
pub(crate) struct MockTxnGenerator {
    account_a: AccountAddress,
    account_a_keypair: AccountKeyPair,
    account_b: AccountAddress,
    account_b_keypair: AccountKeyPair,
}

impl MockTxnGenerator {
    pub fn new() -> Self {
        let (prikey, pubkey) = Self::gen_key_pair();
        let (b_pri, b_pub) = Self::gen_key_pair();
        MockTxnGenerator {
            account_a: AccountAddress::from_public_key(&pubkey),
            account_a_keypair: KeyPair::from(prikey),
            account_b: AccountAddress::from_public_key(&b_pub),
            account_b_keypair: KeyPair::from(b_pri),
        }
    }
    fn gen_key_pair() -> (Ed25519PrivateKey, Ed25519PublicKey) {
        let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
        let seed_buf: [u8; 32] = seed_rng.gen();
        let mut rng = rand::rngs::StdRng::from_seed(seed_buf);

        compat::generate_keypair(rng)
    }

    fn build_mint_txn<E>(
        faucet_seq_number: u64,
        receiver_address: AccountAddress,
        receiver_pubkey: &Ed25519PublicKey,
        amount: u64,
    ) -> Transaction
    where
        E: TransactionExecutor + Sync + Send,
    {
        // add some randomness too.
        let mut rng = rand::thread_rng();
        let amount_to_transfer = rng.gen_range(5000u64, 10000u64);
        // transfer will create account if it not exists.
        let txn = <E as TransactionExecutor>::build_mint_txn(
            receiver_address,
            AuthenticationKey::from_public_key(receiver_pubkey)
                .prefix()
                .to_vec(),
            faucet_seq_number,
            amount,
        );
        txn
    }
    fn get_account_resource(
        account_address: AccountAddress,
        state_db: &dyn ChainStateReader,
    ) -> Result<Option<AccountResource>> {
        // account already exists
        let ap = AccessPath::new_for_account(account_address);
        let account_resource: Option<AccountResource> = match state_db.get(&ap)? {
            None => None,
            Some(b) => Some(b.try_into()?),
        };
        debug!(
            "state: address:{},balance:{:?},seq:{:?}",
            account_address,
            account_resource.as_ref().map(|r| r.balance()),
            account_resource.as_ref().map(|r| r.sequence_number())
        );
        Ok(account_resource)
    }

    pub fn generate_mock_txn<E>(&self, state_db: &dyn ChainStateReader) -> Result<Transaction>
    where
        E: TransactionExecutor + Sync + Send,
    {
        let mint_function = |to_mint_address: AccountAddress, to_mint_key_pair: &AccountKeyPair| {
            let cur_sequence_number = Self::get_account_resource(association_address(), state_db)?
                .expect("association account resource must exists")
                .sequence_number();
            // add some randomness too.
            let mut rng = rand::thread_rng();
            let amount_to_transfer = rng.gen_range(5000u64, 10000u64);

            // transfer will create account if it not exists.
            let transfer_txn = Self::build_mint_txn(
                cur_sequence_number,
                to_mint_address,
                &to_mint_key_pair.public_key,
                amount_to_transfer,
            );
            transfer_txn
        };
        // account already exists
        let account_resource_a = Self::get_account_resource(self.account_a, state_db)?;
        if account_resource_a.is_none() {
            // use faucet to get money
            let mint_txn = mint_function(self.account_a, &self.account_a_keypair);
            return Ok(mint_txn);
        }

        let account_resource_b = Self::get_account_resource(self.account_b, state_db)?;
        if account_resource_b.is_none() {
            // use faucet to get money
            let mint_txn = mint_function(self.account_b, &self.account_b_keypair);
            return Ok(mint_txn);
        }

        let account_resource_a = account_resource_a.unwrap();
        let account_resource_b = account_resource_b.unwrap();

        // A -> B
        if account_resource_a.balance() > 3000 {
            // add some randomness.
            let mut rng = rand::thread_rng();
            let amount_to_transfer = rng.gen_range(1000u64, 5000u64);
            let transfer_txn = <E as TransactionExecutor>::build_transfer_txn(
                self.account_a,
                AuthenticationKey::from_public_key(&self.account_a_keypair.public_key).to_vec(),
                self.account_b,
                AuthenticationKey::from_public_key(&self.account_b_keypair.public_key).to_vec(),
                account_resource_a.sequence_number(),
                amount_to_transfer,
            );
            let signed = transfer_txn
                .sign(
                    &self.account_a_keypair.private_key,
                    self.account_a_keypair.public_key.clone(),
                )?
                .into_inner();
            Ok(Transaction::UserTransaction(signed))
        } else if account_resource_b.balance() > 3000 {
            // B -> A
            // add some randomness.
            let mut rng = rand::thread_rng();
            let amount_to_transfer = rng.gen_range(1000u64, 5000u64);
            let transfer_txn = <E as TransactionExecutor>::build_transfer_txn(
                self.account_b,
                AuthenticationKey::from_public_key(&self.account_b_keypair.public_key).to_vec(),
                self.account_a,
                AuthenticationKey::from_public_key(&self.account_a_keypair.public_key).to_vec(),
                account_resource_b.sequence_number(),
                amount_to_transfer,
            );
            let signed = transfer_txn
                .sign(
                    &self.account_b_keypair.private_key,
                    self.account_b_keypair.public_key.clone(),
                )?
                .into_inner();
            Ok(Transaction::UserTransaction(signed))
        } else {
            // G -> A/B
            let mut rng = rand::thread_rng();
            let to_a = rng.gen_bool(0.5);
            let (to_mint_address, to_mint_keypair) = if to_a {
                (self.account_a, &self.account_a_keypair)
            } else {
                (self.account_b, &self.account_b_keypair)
            };
            mint_function(to_mint_address, to_mint_keypair)
        }
    }
}
