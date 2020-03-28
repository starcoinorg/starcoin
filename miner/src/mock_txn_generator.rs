use anyhow::Result;
use crypto::ed25519::*;
use crypto::test_utils::KeyPair;
use executor::TransactionExecutor;
use logger::prelude::*;
use rand;
use rand::{Rng, SeedableRng};
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

        compat::generate_keypair(Some(&mut rng))
    }

    fn get_account_resource(
        account_address: AccountAddress,
        state_db: &dyn ChainStateReader,
    ) -> Result<Option<AccountResource>> {
        // account already exists
        let ap = AccessPath::new_for_account(account_address);
        let account_resource: Option<AccountResource> = match state_db.get(&ap)? {
            None => None,
            Some(b) => Some(AccountResource::make_from(&b)?),
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
            let faucet_sequence_number =
                Self::get_account_resource(association_address(), state_db)?
                    .expect("association account resource must exists")
                    .sequence_number();
            // add some randomness too.
            let mut rng = rand::thread_rng();
            let amount_to_transfer = rng.gen_range(500000000u64, 1000000000u64);

            // transfer will create account if it not exists.
            let txn = <E as TransactionExecutor>::build_mint_txn(
                to_mint_address,
                AuthenticationKey::from_public_key(&to_mint_key_pair.public_key)
                    .prefix()
                    .to_vec(),
                faucet_sequence_number,
                amount_to_transfer,
            );
            Ok(txn)
        };
        // account already exists
        let account_resource_a = Self::get_account_resource(self.account_a, state_db)?;
        if account_resource_a.is_none() {
            // use faucet to get money
            let mint_txn = mint_function(self.account_a, &self.account_a_keypair);
            return mint_txn;
        }

        let account_resource_b = Self::get_account_resource(self.account_b, state_db)?;
        if account_resource_b.is_none() {
            // use faucet to get money
            let mint_txn = mint_function(self.account_b, &self.account_b_keypair);
            return mint_txn;
        }

        let account_resource_a = account_resource_a.unwrap();
        let account_resource_b = account_resource_b.unwrap();

        let transfer_function = |a: (AccountAddress, &AccountKeyPair),
                                 b: (AccountAddress, &AccountKeyPair),
                                 a_seq_number: u64| {
            // add some randomness.
            let mut rng = rand::thread_rng();
            let amount_to_transfer = rng.gen_range(1000u64, 5000u64);
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
        if account_resource_a.balance() > 3000 {
            transfer_function(
                (self.account_a, &self.account_a_keypair),
                (self.account_b, &self.account_b_keypair),
                account_resource_a.sequence_number(),
            )
        } else if account_resource_b.balance() > 3000 {
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
