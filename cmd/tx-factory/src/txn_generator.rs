use anyhow::Result;
use rand;
use rand::{Rng, SeedableRng};
use starcoin_crypto::ed25519::*;
use starcoin_crypto::test_utils::KeyPair;
use starcoin_executor::TransactionExecutor;
use starcoin_logger::prelude::*;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::{AccountAddress, AuthenticationKey};
use starcoin_types::transaction::Transaction;

type AccountKeyPair = KeyPair<Ed25519PrivateKey, Ed25519PublicKey>;
pub struct MockTxnGenerator {
    account_a: AccountAddress,
    account_a_keypair: AccountKeyPair,
    account_b: AccountAddress,
    account_b_keypair: AccountKeyPair,
    faucet_address: AccountAddress,
}
const BASE_BALANCE: u64 = 500_000_000u64;

impl MockTxnGenerator {
    pub fn new(faucet_address: AccountAddress) -> Self {
        let (prikey, pubkey) = Self::gen_key_pair();
        let (b_pri, b_pub) = Self::gen_key_pair();
        MockTxnGenerator {
            faucet_address,
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

        compat::generate_keypair(Some(&mut rng))
    }

    pub fn generate_mock_txn<E>(&self, state_db: &AccountStateReader) -> Result<Transaction>
    where
        E: TransactionExecutor + Sync + Send,
    {
        let mint_function = |to_mint_address: AccountAddress, to_mint_key_pair: &AccountKeyPair| {
            let account_resource = state_db
                .get_account_resource(&self.faucet_address)?
                .expect("association account resource must exists");
            debug!(
                "faucet: balance:{:?},seq:{:?}",
                account_resource.balance(),
                account_resource.sequence_number()
            );
            // add some randomness too.
            let mut rng = rand::thread_rng();
            let amount_to_transfer = rng.gen_range(BASE_BALANCE, BASE_BALANCE * 2);
            // transfer will create account if it not exists.
            let txn = <E as TransactionExecutor>::build_mint_txn(
                to_mint_address,
                AuthenticationKey::from_public_key(&to_mint_key_pair.public_key)
                    .prefix()
                    .to_vec(),
                account_resource.sequence_number(),
                amount_to_transfer,
            );
            Ok(txn)
        };
        // account already exists
        let account_resource_a = state_db.get_account_resource(&self.account_a)?;
        if account_resource_a.is_none() {
            debug!("account {} not exists, prepare to mint", &self.account_a);
            // use faucet to get money
            let mint_txn = mint_function(self.account_a, &self.account_a_keypair);
            return mint_txn;
        }
        let account_resource_a = account_resource_a.unwrap();
        debug!(
            "address({}): balance:{},seq:{}",
            &self.account_a,
            account_resource_a.balance(),
            account_resource_a.sequence_number()
        );

        let account_resource_b = state_db.get_account_resource(&self.account_b)?;
        if account_resource_b.is_none() {
            debug!("account {} not exists, prepare to mint", &self.account_b);
            // use faucet to get money
            let mint_txn = mint_function(self.account_b, &self.account_b_keypair);
            return mint_txn;
        }

        let account_resource_b = account_resource_b.unwrap();
        debug!(
            "address({}): balance:{},seq:{}",
            &self.account_b,
            account_resource_b.balance(),
            account_resource_b.sequence_number()
        );

        let transfer_function = |a: (AccountAddress, &AccountKeyPair),
                                 b: (AccountAddress, &AccountKeyPair),
                                 a_seq_number: u64| {
            // add some randomness.
            let mut rng = rand::thread_rng();
            let amount_to_transfer = rng.gen_range(300_000, 500_000);
            let transfer_txn = <E as TransactionExecutor>::build_transfer_txn(
                a.0,
                AuthenticationKey::from_public_key(&a.1.public_key).to_vec(),
                b.0,
                AuthenticationKey::from_public_key(&b.1.public_key).to_vec(),
                a_seq_number,
                amount_to_transfer,
            );
            let signed = transfer_txn
                .sign(&a.1.private_key, a.1.public_key.clone())?
                .into_inner();
            Ok(Transaction::UserTransaction(signed))
        };

        // A -> B
        if account_resource_a.balance() > 300_000 + BASE_BALANCE {
            transfer_function(
                (self.account_a, &self.account_a_keypair),
                (self.account_b, &self.account_b_keypair),
                account_resource_a.sequence_number(),
            )
        } else if account_resource_b.balance() > 300_000 + BASE_BALANCE {
            // B -> A
            transfer_function(
                (self.account_b, &self.account_b_keypair),
                (self.account_a, &self.account_a_keypair),
                account_resource_b.sequence_number(),
            )
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
