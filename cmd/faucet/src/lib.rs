use anyhow::Result;
use starcoin_rpc_client::RpcClient;
use starcoin_types::{
    account_address::AccountAddress,
    transaction::{RawUserTransaction, Script, TransactionPayload},
};
use starcoin_wallet_api::{Wallet, WalletAccount};
use std::time::Duration;

struct faucet<W>
where
    W: Wallet,
{
    client: RpcClient,
    faucet_account: WalletAccount,
    wallet: W,
}

impl<W> faucet<W>
where
    W: Wallet,
{
    fn new(faucet_key: Vec<u8>, password: &str, wallet: W, client: RpcClient) -> Self {
        let faucet_account = wallet.import_account(faucet_key, password).unwrap();
        //unlock nerver timeout
        let _ = wallet.unlock_account(faucet_account.address, password, Duration::new(0, 0));
        faucet {
            client,
            faucet_account,
            wallet,
        }
    }

    fn transfer(&self, seq_num: u64, amount: u64, receiver: AccountAddress) -> Result<bool> {
        let raw_tx = trans_tx(self.faucet_account.address, amount, receiver, seq_num);
        match self.wallet.sign_txn(raw_tx) {
            Ok(tx) => self.client.submit_transaction(tx),
            Err(e) => Ok(false),
        }
    }
}

fn trans_tx(
    sender: AccountAddress,
    amount: u64,
    receiver: AccountAddress,
    seq_num: u64,
) -> RawUserTransaction {
    //todo::generate a transfer tx
    RawUserTransaction::new(
        sender,
        seq_num,
        TransactionPayload::Script(Script::default()),
        0,
        0,
        Duration::new(0, 0),
    )
}
