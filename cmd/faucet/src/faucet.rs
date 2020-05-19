use anyhow::{format_err, Result};

use starcoin_executor::executor::Executor;
use starcoin_executor::TransactionExecutor;
use starcoin_rpc_client::{RemoteStateReader, RpcClient};
use starcoin_state_api::AccountStateReader;
use starcoin_types::{account_address::AccountAddress, transaction::RawUserTransaction};
use starcoin_wallet_api::WalletAccount;

pub struct Faucet {
    client: RpcClient,
    faucet_account: WalletAccount,
}

impl Faucet {
    pub fn new(client: RpcClient, faucet_account: WalletAccount) -> Self {
        Faucet {
            client,
            faucet_account,
        }
    }

    pub fn transfer(
        &self,
        amount: u64,
        receiver: AccountAddress,
        auth_key: Vec<u8>,
    ) -> Result<bool> {
        let chain_state_reader = RemoteStateReader::new(&self.client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader
            .get_account_resource(self.faucet_account.address())?
            .ok_or_else(|| {
                format_err!(
                    "Can not find account on chain by address:{}",
                    self.faucet_account.address()
                )
            })?;

        let raw_tx = transfer_tx(
            &self.faucet_account,
            amount,
            receiver,
            account_resource.sequence_number(),
            auth_key,
        );
        let signed_tx = self.client.wallet_sign_txn(raw_tx)?;
        let ret = self.client.submit_transaction(signed_tx)?;
        Ok(ret)
    }
}

fn transfer_tx(
    sender: &WalletAccount,
    amount: u64,
    receiver: AccountAddress,
    seq_num: u64,
    receiver_auth_key_prefix: Vec<u8>,
) -> RawUserTransaction {
    Executor::build_transfer_txn(
        sender.address,
        receiver,
        receiver_auth_key_prefix,
        seq_num,
        amount,
    )
}
