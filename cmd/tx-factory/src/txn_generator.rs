use anyhow::{bail, Result};
use rand;
use rand::{Rng, SeedableRng};
use starcoin_crypto::ed25519::*;
use starcoin_crypto::test_utils::KeyPair;
use starcoin_crypto::ValidKeyStringExt;
use starcoin_executor::TransactionExecutor;
use starcoin_logger::prelude::*;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::{AccountAddress, AuthenticationKey};
use starcoin_types::transaction::{RawUserTransaction, Transaction};
use starcoin_wallet_api::WalletAccount;

type AccountKeyPair = KeyPair<Ed25519PrivateKey, Ed25519PublicKey>;

pub struct MockTxnGenerator {
    receiver_address: AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,

    account: WalletAccount,
}
const BASE_BALANCE: u64 = 500_000_000u64;

impl MockTxnGenerator {
    pub fn new(
        account: WalletAccount,
        receiver_address: AccountAddress,
        receiver_auth_key_prefix: Vec<u8>,
    ) -> Self {
        // let (prikey, pubkey) = Self::gen_key_pair();
        // let (b_pri, b_pub) = Self::gen_key_pair();
        MockTxnGenerator {
            receiver_address,
            receiver_auth_key_prefix,
            account,
        }
    }
    fn gen_key_pair() -> (Ed25519PrivateKey, Ed25519PublicKey) {
        let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
        let seed_buf: [u8; 32] = seed_rng.gen();
        let mut rng = rand::rngs::StdRng::from_seed(seed_buf);

        compat::generate_keypair(Some(&mut rng))
    }

    pub fn generate_mock_txn<E>(&self, state_db: &AccountStateReader) -> Result<RawUserTransaction>
    where
        E: TransactionExecutor + Sync + Send,
    {
        let account_resource = state_db.get_account_resource(self.account.address())?;
        if account_resource.is_none() {
            bail!(
                "account {} not exists, please faucet it",
                self.account.address()
            );
        }
        let account_resource = account_resource.unwrap();

        // TODO: make it configurable
        let amount_to_transfer = 1000;

        if account_resource.balance() <= amount_to_transfer {
            bail!("not enough balance, skip gen mock txn, please faucet it first");
        }

        let transfer_txn = <E as TransactionExecutor>::build_transfer_txn(
            self.account.address,
            AuthenticationKey::from_public_key(&self.account.public_key).to_vec(),
            self.receiver_address,
            self.receiver_auth_key_prefix.clone(),
            account_resource.sequence_number(),
            amount_to_transfer,
        );
        Ok(transfer_txn)
    }
}
