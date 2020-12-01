use anyhow::{format_err, Result};

use starcoin_account_api::AccountInfo;
use starcoin_crypto::ed25519::Ed25519PublicKey;
use starcoin_executor::DEFAULT_EXPIRATION_TIME;
use starcoin_rpc_client::{RemoteStateReader, RpcClient};
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_types::transaction::helpers::get_current_timestamp;
use std::convert::TryFrom;

pub struct Faucet {
    client: RpcClient,
    faucet_account: AccountInfo,
}

const DEFAULT_GAS_PRICE: u64 = 1;
const MAX_GAS: u64 = 10000;

impl Faucet {
    pub fn new(client: RpcClient, faucet_account: AccountInfo) -> Self {
        Faucet {
            client,
            faucet_account,
        }
    }

    pub fn transfer(
        &self,
        amount: u128,
        receiver: AccountAddress,
        public_key: Vec<u8>,
    ) -> Result<()> {
        let chain_state_reader = RemoteStateReader::new(&self.client)?;
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader
            .get_account_resource(self.faucet_account.address())?
            .ok_or_else(|| {
                format_err!(
                    "Can not find account on chain by address:{}",
                    self.faucet_account.address()
                )
            })?;
        let public_key = Ed25519PublicKey::try_from(public_key.as_slice())?;
        let raw_tx = starcoin_executor::build_transfer_txn(
            self.faucet_account.address,
            receiver,
            Some(AuthenticationKey::ed25519(&public_key)),
            account_resource.sequence_number(),
            amount,
            DEFAULT_GAS_PRICE,
            MAX_GAS,
            get_current_timestamp() + DEFAULT_EXPIRATION_TIME,
            self.client.node_info()?.net.chain_id(),
        );
        let signed_tx = self.client.account_sign_txn(raw_tx)?;
        self.client.submit_transaction(signed_tx)?;
        Ok(())
    }
}
